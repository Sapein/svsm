//! VSL Parser for SVSM.
//!
//! This provides a relatively basic parser for SVSM to use
//! to understand it's language (VSL).
//!
//! # Examples
//! ```
//! use std::rc::Rc;
//! let mut parser = svsm::parser::Parser::from_token_list(Rc::from([svsm::lex::Token::String(Rc::from("A string"))]));
//! println!("Output: {:?}" , parser.parse_token());
//! ```

use std::collections::BTreeMap;
use std::fmt::Debug;
use crate::lex::{SmartToken, Token};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::rc::Rc;
use ordered_float::OrderedFloat;
use crate::actions::Action;
use crate::interpreter::{Env, Interpreter};

#[derive(Debug)]
pub struct Parser {
    input: ParserInput,
    parsing_map: bool,
    pos: usize,
}

type Builtin = fn(Vec<Expr>, env: &mut Env) -> Option<Expr>;
type BuiltinMacro = fn(Vec<Expr>, interpreter: &mut Interpreter) -> Option<Expr>;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Expr {
    String(Rc<str>),
    Number(NumberExpr),
    Boolean(bool),
    Symbol(Rc<str>),
    Path(PathBuf),

    VarDecl(Box<Expr>, Box<Expr>),

    GitHubRemote {
        user: Rc<str>,
        repo: Rc<str>,
        branch: Option<Rc<str>>,
    },

    List(Vec<Expr>),
    ListRef(Rc<Expr>, NumberExpr),
    Map(BTreeMap<Expr, Expr>),
    MapRef(Rc<Expr>, Box<Expr>),
    Action(Action),

    FnCall(ExprFnCall),
    FnResult(FnResultExpr),
    
    // Builtins obtain only the scope, it can not manipulate the interpreter state
    Builtin(Builtin),

    // Unlike Builtins, Macros obtain the entire interpreter state and may modify it.
    Macro(BuiltinMacro),
}

impl Expr {
    pub(crate) fn symbol_from_str(str: &str) -> Expr {
        Expr::Symbol(Rc::from(str))
    }

    pub(crate) fn string_from_str(str: &str) -> Expr {
        Expr::String(Rc::from(str))
    }
    
    pub(crate) fn to_string(&self) -> String {
        match self {
            Expr::String(str) => str.to_string(),
            Expr::Number(number) => number.to_string(),
            Expr::Boolean(bool) => bool.to_string(),
            Expr::Symbol(sym) => sym.to_string(),
            Expr::Path(path) => path.to_str().expect("Non utf-8 char").to_string(),
            _ => panic!("Unable to convert to string."),
        }
    }

    pub(crate) fn extract_str(self) -> Rc<str> {
        match self {
            Expr::String(str) => str,
            Expr::Symbol(str) => str,
            _ => panic!("Can't extract str! {:#?}", self),
        }
    }

    pub(crate) fn get_map_value(&self, key: Expr) -> Option<&Expr> {
        match self {
            Expr::Map(map) => map.get(&key),
            _ => None
        }
    }
}

/// This represents a future result of a Function Call that needs to be evaluated.
/// 
/// This is done mostly to allow the interpreter to use lazy evaluation. This works by storing
/// the environment that existed at the time of the Function Call, the arguments, and the exact
/// function. If the interpreter is being run with `disable_lazy` for testing, then this will
/// be immediately evaluated.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct FnResultExpr {
    pub(crate) env: Env,
    pub(crate) args: Vec<Expr>,
    pub(crate) function: Callable,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Callable {
    Builtin(Builtin),
    Macro(BuiltinMacro),
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct NumberExpr {
    pub num: OrderedFloat<f64>,
}

impl NumberExpr {
    pub fn from_number(number: f64) -> Self {
        NumberExpr { num: OrderedFloat::from(number) }
    }
    pub fn to_string(&self) -> String {
        self.num.to_string()
    }
}

impl Hash for NumberExpr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.num.to_be_bytes())
    }
}

#[derive(Debug, PartialOrd, Ord, Eq, Hash, PartialEq, Clone)]
pub struct ExprFnCall {
    pub name: Rc<str>,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
enum ParserInput {
    TokenList(Rc<[Token]>),
    SmartTokenList(Rc<[SmartToken]>),
}

impl Parser {
    pub fn from_token_list(input: Rc<[Token]>) -> Self {
        Parser::new(ParserInput::TokenList(input))
    }

    pub fn from_token_list_smart(input: Rc<[SmartToken]>) -> Self {
        Parser::new(ParserInput::SmartTokenList(input))
    }


    fn is_smarttoken(&self) -> bool {
        match self.input {
            ParserInput::SmartTokenList(_) => true,
            _ => false,
        }
    }


    fn new(input: ParserInput) -> Self {
        Self {
            input,
            parsing_map: false,
            pos: 0,
        }
    }

    fn get_token(&mut self) -> Token {
        match &self.input {
            ParserInput::TokenList(list) => {
                if self.pos >= self.get_input_len() {
                    Token::EoF
                } else {
                    list[self.pos].clone()
                }
            }
            ParserInput::SmartTokenList(list) => {
                if self.pos >= self.get_input_len() {
                    Token::EoF
                } else {
                    list[self.pos].token.clone()
                }
            }
        }
    }

    fn peek_token(&mut self) -> Token {
        self.lookahead_tokens(1)
    }

    fn look_behind(&mut self, count: usize) -> Token {
        match &self.input {
            ParserInput::TokenList(list) => {
                if self.pos - count <= 0 {
                    Token::EoF
                } else {
                    list[self.pos - count].clone()
                }
            }
            ParserInput::SmartTokenList(list) => {
                if self.pos - count <= 0 {
                    Token::EoF
                } else {
                    list[self.pos - count].token.clone()
                }
            }
        }
    }
    
    fn lookahead_tokens(&mut self, count: usize) -> Token {
        match &self.input {
            ParserInput::TokenList(list) => {
                if self.pos + count >= self.get_input_len() {
                    Token::EoF
                } else {
                    list[self.pos + count].clone()
                }
            }
            ParserInput::SmartTokenList(list) => {
                if self.pos + count >= self.get_input_len() {
                    Token::EoF
                } else {
                    list[self.pos + count].token.clone()
                }
            }
        }
    }

    fn peek_next_token_nonws(&mut self, count: usize) -> Token {
        let mut i = count;
        let mut token = self.lookahead_tokens(i);
        while (token == Token::Whitespace) && (token != Token::EoF) {
            i += 1;
            token = self.lookahead_tokens(i);
        }
        token
    }

    fn get_input_len(&self) -> usize {
        match &self.input {
            ParserInput::TokenList(list) => list.len(),
            ParserInput::SmartTokenList(list) => list.len(),
        }
    }

    fn advance(&mut self) {
        match &self.input {
            ParserInput::TokenList(_) => {
                self.pos += 1;
            },
            ParserInput::SmartTokenList(_) => {
                self.pos += 1;
            },
        }
    }

    fn advance_many(&mut self, count: usize) {
        match &self.input {
            ParserInput::TokenList(_) => {
                self.pos += count;
                while self.get_token() == Token::Whitespace {
                    self.pos += 1;
                }
            }
            ParserInput::SmartTokenList(_) => {
                self.pos += count;
                while self.get_token() == Token::Whitespace {
                    self.pos += 1;
                }
            }
        }
    }

    fn advance_skip_whitespace(&mut self) {
        self.advance();
        while self.get_token() == Token::Whitespace && self.pos < self.get_input_len() {
            self.advance();
        }
    }

    pub fn parse_input(&mut self) -> Rc<[Expr]> {
        let mut exprs: Vec<Expr> = vec![];
        while self.pos <= self.get_input_len() && self.get_token() != Token::EoF {
            let expr = match self.parse_token() {
                Some(token) => token,
                None => {
                    self.advance();
                    continue
                },
            };
            exprs.push(expr);
        }
        exprs.into()
    }

    fn parse_path(&mut self) -> Expr {
        let mut path_str = String::new();

        while self.pos < self.get_input_len() {
            let token = self.get_token();
            let input = match token {
                Token::Discard => panic!("Parser got a Discard Token!"),
                Token::String(str) => {
                    let mut string = str.chars().into_iter().collect::<String>();
                    string.remove(0);
                    string.remove(string.len() - 1);
                    Rc::from(string)
                }
                Token::Symbol(str) => str,
                Token::Slash => Rc::from("/"),
                Token::Dot => Rc::from("."),
                _ => break,
            };
            path_str.push_str(&input);
            self.advance();
        }

        self.pos -= 1;
        Expr::Path(PathBuf::from(path_str))
    }

    fn parse_list(&mut self) -> Expr {
        let mut list: Vec<Expr> = Vec::new();

        while self.pos < self.get_input_len() {
            self.advance();
            let expr = match self.get_token() {
                Token::CloseBracket => {
                    self.advance();
                    break;
                }
                Token::Comma => continue,
                Token::Whitespace => continue,
                _ => self.parse_token(),
            }
            .unwrap();
            list.push(expr);
        }
        self.pos -= 1;
        Expr::List(list)
    }

    fn get_token_position(&self) -> (usize, Option<(usize, usize)>) {
        match &self.input {
            ParserInput::TokenList(_) => {
                (self.pos, None)
            },
            ParserInput::SmartTokenList(list) => {
                let token = list[self.pos].clone();
                (token.row, Some(token.col))
            }
        }
    }

    fn peek_discard_whitespace(&self) -> Token {
        let mut count: usize = 1;
        loop {
            match &self.input {
                ParserInput::TokenList(list) => {
                    if self.pos + count >= self.get_input_len() {
                        return Token::EoF;
                    } else {
                        let token = list[self.pos + count].clone();
                        match token {
                            Token::Whitespace => {
                                count += 1;
                                continue;
                            }
                            _ => return token,
                        }
                    }
                },
                ParserInput::SmartTokenList(list) => {
                    if self.pos + count >= self.get_input_len() {
                        return Token::EoF;
                    } else {
                        let token = list[self.pos + count].token.clone();
                        match token {
                            Token::Whitespace => {
                                count += 1;
                                continue;
                            }
                            _ => return token,
                        }
                    }
                }
            }
        }
    }

    fn parse_map(&mut self) -> Expr {
        self.parsing_map = true;
        let mut map: BTreeMap<Expr, Expr> = BTreeMap::new();

        while self.pos < self.get_input_len() || self.get_token() != Token::EoF {
            self.advance();
            let expr = match self.get_token() {
                Token::CloseBrace => {
                    self.advance();
                    break;
                }
                Token::Semicolon => {
                    continue
                }
                Token::Symbol(sym) if self.peek_discard_whitespace() == Token::Equal => {
                    self.advance_skip_whitespace();
                    self.advance_skip_whitespace();
                    let token = self.parse_token();
                    match token {
                        None => continue,
                        Some(t) => (Expr::Symbol(sym), t)
                    }
                }
                Token::Whitespace => continue,
                Token::Comma => continue,
                Token::CloseBracket => {
                    break;
                }
                _ => {
                    if self.is_smarttoken() {
                        let (row, col) = self.get_token_position();
                        let (col_start, col_end) = col.unwrap();
                        panic!("Unknown symbol {:?} ({}), at key position in map at row {}, column: ({}, {})", self.get_token(), self.get_token().get_token(), row, col_start, col_end)
                    }
                    panic!("Unknown symbol at key position in map!")
                }
            };
            if map.contains_key(&expr.1) {
                match expr.1 {
                    Expr::Symbol(str) => if self.is_smarttoken() {
                        let (row, col) = self.get_token_position();
                        let (col_start, col_end) = col.unwrap();
                        let Expr::Symbol(map_name) = expr.0 else { panic!("This should never happen!") };
                        panic!("Key {} already exists in Map {}. New definition at row {}, column: ({}, {})", str, map_name, row, col_start, col_end);
                    } else {
                        panic!("Key {} already exists in map!", str)
                    },
                    _ => panic!()
                }
            }
            map.insert(expr.0, expr.1);
        }

        self.pos -= 1;
        self.parsing_map = false;
        Expr::Map(map)
    }

    pub fn parse_parens(&mut self) -> Vec<Expr> {
        let mut exprs: Vec<Expr> = Vec::new();

        self.advance();
        while self.pos < self.get_input_len() {
            let expr = match self.get_token() {
                Token::CloseParen => break,
                _ => {
                    let token = self.parse_token();
                    match token {
                        Some(token) => token,
                        None => continue,
                    }
                }
            };
            exprs.push(expr);
        }

        exprs
    }

    fn parse_fncall(&mut self, name: Rc<str>) -> Expr {
        let mut args: Vec<Expr> = Vec::new();

        while self.pos < self.get_input_len() {
            self.advance();
            let expr = match self.get_token() {
                Token::Discard => panic!("Parser got a Discard Token!"),
                Token::Comma | Token::Semicolon | Token::EoF | Token::CloseParen | Token::CloseBrace | Token::CloseBracket => break,
                Token::Equal => Expr::Symbol(Rc::from("=")),
                Token::Whitespace =>  continue,
                Token::OpenParen => {
                    args.extend(self.parse_parens());
                    continue;
                }

                _ => match self.parse_token() {
                    Some(token) => token,
                    None => continue,
                },
            };
            args.push(expr);
        }

        Expr::FnCall(ExprFnCall { name, args })
    }

    fn parse_assignment(&mut self, symbol: Expr) -> Expr {
        self.advance_skip_whitespace();
        Expr::VarDecl(Box::from(symbol), Box::from(self.parse_token().unwrap()))
    }
    

    fn parse_symbol(&mut self, symbol: Rc<str>) -> Expr {
        self.advance_skip_whitespace();
        match self.get_token() {
            Token::Semicolon | Token::Comma | Token::CloseBrace | Token::CloseBracket | Token::EoF => Expr::Symbol(symbol),
            Token::Equal if self.parsing_map => Expr::Symbol(symbol),
            Token::Equal => {
                self.parse_assignment(Expr::Symbol(symbol))
            }
            Token::Dot => {
                let map_attr =  match self.peek_token() {
                    Token::Symbol(attr) => attr,
                    Token::Number(i) => {
                        if self.is_smarttoken() {
                            let (row, col) = self.get_token_position();
                            let col = col.unwrap();
                            panic!("Attempt to index a map {} with a number {} at row {}, column ({}, {})", symbol, i, row, col.0, col.1)
                        }
                        panic!("You can not index a Map with a number!");
                    },
                    Token::Slash => {
                        self.pos -= 1;
                        return self.parse_fncall(symbol);
                    }
                    _ => {
                        if self.is_smarttoken() {
                            let (row, col) = self.get_token_position();
                            let col = col.unwrap();
                            panic!("Malformed Mapref at row {}, column ({}, {}).\nMap Name: {}\nAttribute: {:?}", row, col.0, col.1, symbol, self.peek_token())
                        }
                        panic!("Malformed MapRef!")
                    },
                };

                let map_ref = self.parse_mapref(symbol, map_attr);
                match self.peek_next_token_nonws(0) {
                    Token::Equal => {
                        self.parse_assignment(map_ref)
                    }
                    _ => map_ref,
                }
            },
            Token::OpenBracket if self.look_behind(1) != Token::Whitespace => {
                let list_ref = match self.peek_token() {
                    Token::Number(i) if self.lookahead_tokens(2) == Token::CloseBracket => self.parse_listref(symbol, i),
                    Token::Number(i) if self.lookahead_tokens(2) != Token::Comma => panic!("Malformed List or ListRef! {}[{}", symbol, i),
                    Token::CloseBracket => {
                        self.pos -= 1;
                        return self.parse_fncall(symbol);
                    }
                    _ if self.lookahead_tokens(2) != Token::Comma => panic!("Malformed List or ListRef! {}. Peeked: {:?} ; Lookahead: {:?}", symbol, self.peek_token(), self.lookahead_tokens(2)),
                    _ => panic!("List panic!"),
                };

                match self.peek_next_token_nonws(1) {
                    Token::Equal => {
                        self.advance_skip_whitespace();
                        self.parse_assignment(list_ref)
                    }
                    _ => {
                        list_ref
                    },
                }
            }
            _ => {
                self.pos -= 1;
                let res = self.parse_fncall(symbol.clone());
                res
            },
        }
    }

    fn parse_mapref(&mut self, map_symbol: Rc<str>, index_symbol: Rc<str>) -> Expr {
        self.advance_many(2);
        Expr::MapRef(
            Rc::from(Expr::Symbol(map_symbol)),
            Box::from(Expr::Symbol(index_symbol)),
        )
    }

    fn parse_listref(&mut self, list_symbol: Rc<str>, index: f64) -> Expr {
        if index.fract() != 0.0 {
            if self.is_smarttoken() {
                let (row, col) = self.get_token_position();
                let col = col.unwrap();
                panic!("Attempt to index a list by non-integer number {} at row {}, column {:?}", index, row, col)
            }
            panic!("Can not index a list by a non-integer number! Number: {}", index);
        }
        self.advance_many(3);
        Expr::ListRef(
            Rc::from(Expr::Symbol(list_symbol)),
            NumberExpr { num: OrderedFloat::from(index) },
        )
    }

    pub fn parse_token(&mut self) -> Option<Expr> {
        match self.get_token() {
            Token::Discard => panic!("Parser got a Discard Token!"),
            Token::Boolean(b) => Some(Expr::Boolean(b)),
            Token::String(str) => Some(Expr::String(str)),
            Token::Number(num) => Some(Expr::Number(NumberExpr { num: OrderedFloat::from(num) })),
            Token::Slash => Some(self.parse_path()),
            Token::Dot if self.peek_token() == Token::Slash => Some(self.parse_path()),
            Token::OpenBracket => Some(self.parse_list()),
            Token::OpenBrace => Some(self.parse_map()),
            Token::OpenParen => self.parse_parens().iter().map(|e| { e.to_owned() }).nth(1),
            Token::Symbol(sym) => Some(self.parse_symbol(sym)),
            Token::CloseBrace => None,
            Token::CloseParen => None,
            Token::CloseBracket => None,
            Token::Semicolon => None,
            Token::EoF => None,
            Token::Whitespace => {
                self.advance_skip_whitespace();
                self.parse_token()
            }
            _ => {
                if self.is_smarttoken() {
                    let (row, col) = self.get_token_position();
                    panic!("Unknown token: {:?} at row {:?}, column: {:?}", self.get_token(), row, col.unwrap());
                }
                panic!("Unknown token! {:?}", self.get_token())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_string_parse() {
        let test_input = [
            "'This is a string'",
            "\"This is also a string\"",
            "This is also \
         a string",
        ];

        for input in test_input {
            let output = Parser::new(ParserInput::TokenList(Rc::new([Token::String(Rc::from(
                input,
            ))])))
            .parse_token()
            .unwrap();
            assert_eq!(output, Expr::String(Rc::from(input)));
        }
    }
    #[test]
    pub fn test_bool_parse() {
        let test_input = [true, false];
        for input in test_input {
            let output = Parser::new(ParserInput::TokenList(Rc::new([Token::Boolean(input)])))
                .parse_token()
                .unwrap();
            assert_eq!(output, Expr::Boolean(input));
        }
    }
    #[test]
    pub fn test_number_parse() {
        let test_input = [0.1, 1.0, 1.1, 1.01231];
        for input in test_input {
            let output = Parser::new(ParserInput::TokenList(Rc::new([Token::Number(input)])))
                .parse_token()
                .unwrap();
            assert_eq!(output, Expr::Number(NumberExpr { num: OrderedFloat::from(input) }));
        }
    }
    #[test]
    pub fn test_symbol_parse() {
        let test_input = ["Symbol", "AnotherSymbol"];
        for input in test_input {
            let output = Parser::new(ParserInput::TokenList(Rc::new([Token::Symbol(Rc::from(
                input,
            ))])))
            .parse_token()
            .unwrap();
            assert_eq!(output, Expr::Symbol(Rc::from(input)));
        }
    }

    #[test]
    pub fn test_fncall_parse() {
        let test_input: Vec<Rc<[Token]>> = vec![
            Rc::from([Token::Symbol(Rc::from("print")), Token::Whitespace, Token::Number(1.0),]),
            Rc::from([Token::Symbol(Rc::from("print")), Token::OpenParen, Token::Symbol(Rc::from("add")), Token::Number(1.0), Token::Number(2.0), Token::CloseParen,]),
            Rc::from([Token::Symbol(Rc::from("print")), Token::Whitespace, Token::OpenBrace, Token::CloseBrace,]),
            Rc::from([Token::Symbol(Rc::from("print")), Token::Whitespace, Token::OpenBracket, Token::CloseBracket,]),
        ];

        let test_output = [
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) })],
            }),
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::FnCall(ExprFnCall {
                    name: Rc::from("add"),
                    args: vec![
                        Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) }),
                        Expr::Number(NumberExpr { num: OrderedFloat::from(2.0) }),
                    ],
                })],
            }),
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::Map(BTreeMap::new())],
            }),
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::List(vec![])],
            }),
        ];
        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }


    #[test]
    pub fn test_assignment() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([Token::Symbol(Rc::from("test")), Token::Equal, Token::Number(1.0)])];
        let expected_output = vec![Expr::VarDecl(Box::from(Expr::Symbol(Rc::from("test"))), Box::from(Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) })))];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, expected_output[i]);
        }
    }

    #[test]
    pub fn test_path_parse() {
        let test_input: Vec<Rc<[Token]>> = vec![
            Rc::from([Token::Dot, Token::Slash, Token::Symbol(Rc::from("test"))]),
            Rc::from([Token::Slash]),
            Rc::from([Token::Slash, Token::Symbol(Rc::from("root"))]),
            Rc::from([
                Token::Slash,
                Token::Symbol(Rc::from("root")),
                Token::Slash,
                Token::String(Rc::from("'a path'")),
            ]),
            Rc::from([Token::Slash, Token::Whitespace, Token::Slash]),
            Rc::from([Token::Slash, Token::Number(10.0)]),
            Rc::from([
                Token::Dot,
                Token::Slash,
                Token::Dot,
                Token::Symbol(Rc::from("test")),
                Token::Dot,
                Token::Symbol(Rc::from("txt")),
            ]),
        ];

        let test_output = [
            Expr::Path(PathBuf::from("./test")),
            Expr::Path(PathBuf::from("/")),
            Expr::Path(PathBuf::from("/root")),
            Expr::Path(PathBuf::from("/root/a path")),
            Expr::Path(PathBuf::from("/")),
            Expr::Path(PathBuf::from("/")),
            Expr::Path(PathBuf::from("./.test.txt")),
        ];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }
    #[test]
    pub fn test_list_parse() {
        let test_input: Vec<Rc<[Token]>> = vec![
            Rc::from([Token::OpenBracket, Token::CloseBracket]),
            Rc::from([
                Token::OpenBracket,
                Token::OpenBracket,
                Token::CloseBracket,
                Token::Comma,
                Token::CloseBracket,
            ]),
            Rc::from([
                Token::OpenBracket,
                Token::Number(1.0),
                Token::Comma,
                Token::Symbol(Rc::from("test")),
                Token::Comma,
                Token::CloseBracket,
            ]),
        ];

        let test_output = [
            Expr::List(vec![]),
            Expr::List(vec![Expr::List(vec![])]),
            Expr::List(vec![
                Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) }),
                Expr::Symbol(Rc::from("test")),
            ]),
        ];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }

    #[test]
    pub fn test_map_parse() {
        let test_input: Vec<Rc<[Token]>> = vec![
            Rc::from([Token::OpenBrace, Token::CloseBrace]),
            Rc::from([
                Token::OpenBrace,
                Token::Symbol(Rc::from("test")),
                Token::Equal,
                Token::Number(1.0),
                Token::Semicolon,
                Token::CloseBrace,
            ]),
            Rc::from([Token::OpenBrace,
                            Token::Symbol(Rc::from("test")), Token::Equal, Token::OpenBrace,
                                Token::Symbol(Rc::from("id")), Token::Equal, Token::Number(1.0), Token::Semicolon,
                            Token::CloseBrace, Token::Semicolon,
                          Token::CloseBrace]),
        ];

        let test_output = [
            Expr::Map(BTreeMap::new()),
            Expr::Map(BTreeMap::from([(
                Expr::Symbol(Rc::from("test")),
                 Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) }),
            )])),
            Expr::Map(BTreeMap::from([(
                Expr::Symbol(Rc::from("test")),
                Expr::Map(BTreeMap::from([(
                    Expr::Symbol(Rc::from("id")),
                    Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) }),
                )])),
            )])),
        ];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }

    #[test]
    pub fn test_listref() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::OpenBracket,
            Token::Number(1.0),
            Token::CloseBracket,
        ])];

        let test_output = [Expr::ListRef(
            Rc::from(Expr::Symbol(Rc::from("test"))),
            NumberExpr { num: OrderedFloat::from(1.0) },
        )];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }

    #[test]
    pub fn test_mapref() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Dot,
            Token::Symbol(Rc::from("test")),
        ])];

        let test_output = [Expr::MapRef(
            Rc::from(Expr::Symbol(Rc::from("test"))),
            Box::from(Expr::Symbol(Rc::from("test"))),
        )];

        for (i, input) in test_input.into_iter().enumerate() {
            let output = Parser::new(ParserInput::TokenList(input))
                .parse_token()
                .unwrap();
            assert_eq!(output, test_output[i]);
        }
    }

    #[test]
    #[should_panic]
    pub fn test_bad_listref() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Whitespace,
            Token::OpenBracket,
            Token::Number(1.0),
            Token::CloseBrace,
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic]
    pub fn test_bad_listref_unclosed_brace() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::OpenBracket,
            Token::Number(1.0),
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic(expected = "Malformed List or ListRef!")]
    pub fn test_bad_listref_symbol() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::OpenBracket,
            Token::Symbol(Rc::from("test")),
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic(expected = "Can not index a list by a non-integer number!")]
    pub fn test_bad_listref_fractional() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::OpenBracket,
            Token::Number(1.1),
            Token::CloseBracket,
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic]
    pub fn test_bad_mapref() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Dot,
            Token::Whitespace,
            Token::Symbol(Rc::from("test")),
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic(expected = "You can not index a Map with a number!")]
    pub fn test_bad_mapref_number() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Dot,
            Token::Number(1.0),
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic(expected = "Malformed MapRef!")]
    pub fn test_bad_mapref_bool() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Dot,
            Token::Boolean(true),
        ])];

        for (_, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic]
    pub fn test_discard_parse() {
        Parser::new(ParserInput::TokenList(Rc::new([Token::Discard]))).parse_token();
    }
}
