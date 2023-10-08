use crate::lex::Token;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::rc::Rc;

pub struct Parser {
    input: ParserInput,
    parsing_map: bool,
    pos: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expr {
    String(Rc<str>),
    Number(NumberExpr),
    Boolean(bool),
    Symbol(Rc<str>),
    Path(PathBuf),

    VarDecl(Rc<Expr>, Rc<Expr>),

    List(Vec<Expr>),
    ListRef(Rc<Expr>, NumberExpr),
    Map(Vec<MapAttrExpr>),
    MapRef(Rc<Expr>, Rc<Expr>),

    FnCall(ExprFnCall),
    EoF,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NumberExpr {
    num: f64,
}

impl NumberExpr {
    pub fn from_number(number: f64) -> Self {
        NumberExpr { num: number }
    }
}

impl Hash for NumberExpr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.num.to_be_bytes())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MapAttrExpr {
    key: Expr,
    value: Expr,
}

impl MapAttrExpr {
    pub fn new(key: Expr, value: Expr) -> Self {
        match key {
            Expr::Symbol(_) => (),
            _ => panic!("Key *must* be a symbol!"),
        };

        Self { key, value }
    }
}

impl Eq for NumberExpr {}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ExprFnCall {
    name: Rc<str>,
    args: Vec<Expr>,
}

enum ParserInput {
    TokenList(Rc<[Token]>),
}

impl Parser {
    pub fn from_token_list(input: Rc<[Token]>) -> Self {
        Parser::new(ParserInput::TokenList(input))
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
        }
    }

    fn peek_token(&mut self) -> Token {
        self.lookahead_tokens(1)
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
        }
    }

    fn peek_next_token(&mut self) -> Token {
        self.lookahead_tokens(2)
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
        }
    }

    fn advance(&mut self) {
        match &self.input {
            ParserInput::TokenList(_) => {
                self.pos += 1;
            }
        }
    }

    fn advance_many(&mut self, count: usize) {
        match &self.input {
            ParserInput::TokenList(_) => {
                self.pos += count;
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
                Some(T) => T,
                None => continue,
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

        Expr::List(list)
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
                }
            }
        }
    }

    fn parse_map(&mut self) -> Expr {
        self.parsing_map = true;
        let mut map: Vec<MapAttrExpr> = Vec::new();

        while self.pos < self.get_input_len() || self.get_token() != Token::EoF {
            self.advance();
            let expr = match self.get_token() {
                Token::CloseBrace => {
                    self.advance();
                    break;
                }
                Token::Semicolon => continue,
                Token::Symbol(sym) if self.peek_discard_whitespace() == Token::Equal => {
                    let key = Expr::Symbol(sym);
                    self.advance_skip_whitespace();
                    self.advance_skip_whitespace();
                    let value = self.parse_token().unwrap();
                    MapAttrExpr { key, value }
                }
                Token::Whitespace => continue,
                _ => panic!("Unknown symbol at key position in map!"),
            };
            map.push(expr);
        }

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
                        Some(T) => T,
                        None => continue,
                    }
                }
            };
            self.advance();
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
                Token::Comma | Token::Semicolon | Token::EoF => break,
                Token::Equal => Expr::Symbol(Rc::from("=")),
                Token::Whitespace => continue,
                Token::OpenParen => {
                    args.extend(self.parse_parens());
                    continue;
                }

                _ => match self.parse_token() {
                    Some(T) => T,
                    None => continue,
                },
            };
            args.push(expr);
        }

        Expr::FnCall(ExprFnCall { name, args })
    }

    fn parse_assignment(&mut self, symbol: Expr) -> Expr {
        self.advance_skip_whitespace();
        Expr::VarDecl(Rc::from(symbol), Rc::from(self.parse_token().unwrap()))
    }

    fn parse_symbol(&mut self, symbol: Rc<str>) -> Expr {
        match self.peek_token() {
            Token::Semicolon | Token::Comma | Token::EoF => Expr::Symbol(symbol),
            Token::Equal if self.parsing_map => Expr::Symbol(symbol),
            Token::Equal if ! self.parsing_map => {
                self.advance_skip_whitespace();
                self.parse_assignment(Expr::Symbol(symbol))
            }
            Token::Dot => match self.peek_next_token() {
                Token::Symbol(map) => match self.peek_next_token_nonws(3) {
                    Token::Equal => {
                        let mapref = self.parse_mapref(symbol, map);
                        self.advance_skip_whitespace();
                        self.parse_assignment(mapref)
                    }
                    _ => self.parse_mapref(symbol, map)
                }
                Token::Number(_) => panic!("You can not index a Map with a number!"),
                _ => self.parse_fncall(symbol),
            },
            Token::OpenBracket => match self.peek_next_token() {
                Token::Number(i) if self.lookahead_tokens(3) == Token::CloseBracket => {
                    self.parse_listref(symbol, i)
                }
                Token::Number(i) if self.lookahead_tokens(3) != Token::Comma => {
                    panic!("Malformed List or ListRef! {}[{}", symbol, i)
                }
                _ if self.lookahead_tokens(3) != Token::Comma => {
                    panic!("Malformed List or ListRef!")
                }
                _ => self.parse_fncall(symbol),
            },
            _ => self.parse_fncall(symbol),
        }
    }

    fn parse_mapref(&mut self, map_symbol: Rc<str>, index_symbol: Rc<str>) -> Expr {
        self.advance_many(3);
        Expr::MapRef(
            Rc::from(Expr::Symbol(map_symbol)),
            Rc::from(Expr::Symbol(index_symbol)),
        )
    }

    fn parse_listref(&mut self, list_symbol: Rc<str>, index: f64) -> Expr {
        if index.fract() != 0.0 {
            panic!("Can not index a list by a non-integer number!")
        }
        self.advance_many(4);
        Expr::ListRef(
            Rc::from(Expr::Symbol(list_symbol)),
            NumberExpr { num: index },
        )
    }

    pub fn parse_token(&mut self) -> Option<Expr> {
        match self.get_token() {
            Token::Discard => panic!("Parser got a Discard Token!"),
            Token::Boolean(b) => Some(Expr::Boolean(b)),
            Token::String(str) => Some(Expr::String(str)),
            Token::Number(num) => Some(Expr::Number(NumberExpr { num })),
            Token::Slash => Some(self.parse_path()),
            Token::Dot if self.peek_token() == Token::Slash => Some(self.parse_path()),
            Token::OpenBracket => Some(self.parse_list()),
            Token::OpenBrace => Some(self.parse_map()),
            Token::Symbol(sym) => Some(self.parse_symbol(sym)),
            Token::CloseParen => None,
            Token::EoF => None,
            Token::Whitespace => {
                self.advance_skip_whitespace();
                self.parse_token()
            }
            _ => panic!("Unknown token! {:?}", self.get_token()),
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
                input.clone(),
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
            assert_eq!(output, Expr::Number(NumberExpr { num: input }));
        }
    }
    #[test]
    pub fn test_symbol_parse() {
        let test_input = ["Symbol", "AnotherSymbol"];
        for input in test_input {
            let output = Parser::new(ParserInput::TokenList(Rc::new([Token::Symbol(Rc::from(
                input.clone(),
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
                args: vec![Expr::Number(NumberExpr { num: 1.0 })],
            }),
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::FnCall(ExprFnCall {
                    name: Rc::from("add"),
                    args: vec![
                        Expr::Number(NumberExpr { num: 1.0 }),
                        Expr::Number(NumberExpr { num: 2.0 }),
                    ],
                })],
            }),
            Expr::FnCall(ExprFnCall {
                name: Rc::from("print"),
                args: vec![Expr::Map(vec![])],
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
        let expected_output = vec![Expr::VarDecl(Rc::from(Expr::Symbol(Rc::from("test"))), Rc::from(Expr::Number(NumberExpr { num: 1.0 })))];

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
                Expr::Number(NumberExpr { num: 1.0 }),
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
            // Rc::from([Token::OpenBrace,
            //                 Token::Symbol(Rc::from("test")), Token::Equal, Token::OpenBrace,
            //                     Token::Symbol(Rc::from("id")), Token::Equal, Token::Number(1.0), Token::Semicolon,
            //                 Token::CloseBrace, Token::Semicolon,
            //               Token::CloseBrace]),
        ];

        let test_output = [
            Expr::Map(vec![]),
            Expr::Map(vec![MapAttrExpr {
                key: Expr::Symbol(Rc::from("test")),
                value: Expr::Number(NumberExpr { num: 1.0 }),
            }]),
            Expr::Map(vec![MapAttrExpr {
                key: Expr::Symbol(Rc::from("test")),
                value: Expr::Map(vec![MapAttrExpr {
                    key: Expr::Symbol(Rc::from("id")),
                    value: Expr::Number(NumberExpr { num: 1.0 }),
                }]),
            }]),
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
            NumberExpr { num: 1.0 },
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
            Rc::from(Expr::Symbol(Rc::from("test"))),
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

        for (i, input) in test_input.into_iter().enumerate() {
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

        for (i, input) in test_input.into_iter().enumerate() {
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

        for (i, input) in test_input.into_iter().enumerate() {
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

        for (i, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic]
    pub fn test_bad_mapref() {
        let test_input: Vec<Rc<[Token]>> = vec![Rc::from([
            Token::Symbol(Rc::from("test")),
            Token::Whitespace,
            Token::Dot,
            Token::Symbol(Rc::from("test")),
        ])];

        for (i, input) in test_input.into_iter().enumerate() {
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

        for (i, input) in test_input.into_iter().enumerate() {
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

        for (i, input) in test_input.into_iter().enumerate() {
            Parser::new(ParserInput::TokenList(input)).parse_token();
        }
    }

    #[test]
    #[should_panic]
    pub fn test_discard_parse() {
        Parser::new(ParserInput::TokenList(Rc::new([Token::Discard]))).parse_token();
    }
}
