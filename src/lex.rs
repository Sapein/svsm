//! VSL Lexer for SVSM.
//!
//! This provides a relatively basic lexer for SVSM to use
//! to understand it's language (VSL).
//!
//! # Examples
//! ```
//! let mut lexer = svsm::lex::Lexer::new("'A test'".chars().collect());
//! println!("Output: {:?}" , lexer.tokenize_input())
//! ```

use std::ops::Add;
use std::rc::Rc;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WHITESPACE: Regex = Regex::new("\\s").unwrap();
    static ref BREAKING: Regex = Regex::new("\\s|\\{|\\}|;|,|\\[|\\]|=|\\(|\\)").unwrap();
    static ref VALID_SYMBOL: Regex = Regex::new("^[A-Za-z_]+(?:[A-Za-z_0-9]|[A-Za-z_0-9\\-][A-Za-z_0-9]+)*$").unwrap();
}

/// A Lexer is represented here.
#[derive(Debug)]
pub struct Lexer {
    pub discard_whitespace: bool,
    pub discard_eof: bool,
    input: Vec<char>,
    pos: usize,

    row: usize,
    col: usize,

    trow: usize,
    tcol: usize,
    tpos: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SmartToken {
    pub row: usize,
    pub col: (usize, usize),
    pub token: Token,
}

/// Representation of a valid Token
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(Rc<str>),
    Boolean(bool),
    Number(f64),
    Symbol(Rc<str>),
    Semicolon,
    Comma,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Equal,
    Dot,
    Slash,
    Whitespace,
    EoF,

    /// An 'empty' Token that may be generated.
    Discard,
}


impl Lexer {

    pub fn from_string(input: &str) -> Self {
        Lexer::new(input.chars().collect())
    }


    /// Returns a new Lexer with the input given.
    ///
    /// # Arguments
    /// * `input` - A Vector of characters to tokenize.
    pub fn new(input: Vec<char>) -> Self {
        Self {
            discard_whitespace: false,
            discard_eof: false,
            input,
            pos: 0,

            col: 1,
            row: 1,

            tcol: 1,
            trow: 1,
            tpos: 1
        }
    }

    pub fn toggle_whitespace(mut self) -> Self {
        self.discard_whitespace = !self.discard_whitespace;
        self
    }

    /// Looks at the next character.
    fn peek(&self) -> char {
        if self.pos + 1 >= self.input.len() {
            return '\0';
        }
        self.input[self.pos + 1]
    }

    /// Looks at the next character, but as a String.
    fn peek_str(&self) -> String {
        String::from(self.peek())
    }

    /// Get the current character, or the null character if there is none left.
    fn get_char(&self) -> char {
        if self.pos >= self.input.len() {
            return '\0';
        }
        self.input[self.pos]
    }

    /// Get the current charater, but as a string.
    fn get_str(&self) -> String {
       String::from(self.get_char())
    }

    /// Advance the current lexer position by one character.
    fn advance(&mut self) -> &Self {
        self.pos += 1;
        self.col += 1;
        if self.get_char() == '\n' {
            self.row += 1;
            self.col = 1;
        }
        self
    }

    /// Advance the current position, and get the new character.
    fn next(&mut self) -> char {
        self.advance();
        self.get_char()
    }

    /// Collect all characters into one vector until the pattern matches, including the character that made the match.
    ///
    /// # Arguments
    /// * `pattern` - A Regex Pattern to match.
    fn collect_to(&mut self, pattern: &Regex) -> Vec<char> {
        let mut tokens: Vec<char> = vec!(self.get_char());
        self.advance();
        while !pattern.is_match(&self.get_str()) && self.peek() != '\0'{
            tokens.push(self.get_char());
            self.advance();
        }
        tokens.push(self.get_char());
        tokens
    }

    /// Keep moving forward -- discarding input -- until we reach the pattern or end of input.
    ///
    /// # Arguments
    /// * `pattern` - A Regex Pattern to match.
    fn advance_until(&mut self, pattern: &Regex) -> &Self {
        while !pattern.is_match(&self.peek_str()) && self.peek() != '\0' {
            self.advance();
        }
        self
    }

    /// Collect the input while the pattern matches.
    ///
    /// # Arguments
    /// * `pattern`  - A Regex pattern to match on.
    fn collect_while(&mut self, pattern: &Regex) -> Vec<char> {
        let mut token: Vec<char> = vec!();
        loop {
            if pattern.is_match(self.get_str().as_str()) && self.get_char() != '\0' {
                token.push(self.get_char());
            } else if self.peek() != '\0' && pattern.is_match(token.iter().collect::<String>().add(self.get_str().as_str()).as_str()) {
                token.push(self.get_char());
            } else if self.peek() != '\0' && pattern.is_match(token.iter().collect::<String>().add(self.get_str().as_str()).add(self.peek_str().as_str()).as_str()) {
                token.push(self.get_char());
            } else {
                break;
            }
            self.advance();
        }
        token
    }

    fn backup(&mut self) {
        if self.pos > 0 {
            match self.pos.checked_sub(1) {
                Some(i) => self.pos = i,
                None => ()
            }
        }
    }

    pub fn tokenize_input(&mut self) -> Rc<[Token]> {
        let mut tokens: Vec<Token> = vec!();
        while self.pos <= self.input.len() {
            let token = self.next_token();
            match token {
                Token::Discard => (),
                _ => tokens.push(token),
            }
        }
        tokens.into()
    }

    /// Collect and tokenize the entirety of the input in one go.
    pub fn tokenize_input_smart(&mut self) -> Rc<[SmartToken]> {
        let mut tokens: Vec<SmartToken> = vec!();
        while self.pos <= self.input.len() {
            let token = self.next_token();
            match token {
                Token::Discard => (),
                _ => {
                    tokens.push(SmartToken {
                        row: self.trow,
                        col: (self.tcol, self.col),
                        token
                    })
                }
            }
        }
        tokens.into()
    }

    pub fn location(&self) -> (usize, usize) {
        (self.trow, self.tcol)
    }

    #[allow(dead_code)]
    fn peek_token(&mut self) -> Token {
        let token = self.next_token();
        self.pos = self.tpos;
        self.row = self.trow;
        self.col = self.tcol;
        token
    }

    /// Gets the next token in the input.
    pub(crate) fn next_token(&mut self) -> Token {
        self.tpos = self.pos;
        self.tcol = self.col;
        self.trow = self.row;
        let token = match self.get_char() {
            '#' => {
                self.advance_until(&Regex::new("\\n").unwrap());
                Token::Discard
            },
            '\'' => {
                let (row, col) = (self.row, self.col);
                let result = self.collect_to(&Regex::new("'").unwrap());
                if result.last().unwrap() != &'\'' {
                    panic!("String opened on line {}, char {} not closed until end of file!\n String: {}", row, col, result.iter().collect::<String>());
                }
                Token::String(result.iter().collect::<String>().into())
            },
            '"' => {
                let (row, col) = (self.row, self.col);
                let result = self.collect_to(&Regex::new("\"").unwrap());
                if result.last().unwrap() != &'"' {
                    panic!("String opened on line {}, char {} not closed until end of file!\n String: {}", row, col, result.iter().collect::<String>());
                }
                Token::String(result.iter().collect::<String>().into())
            }

            't' => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.iter().collect::<String>() == "true" {
                    Token::Boolean(true)
                } else {
                    self.backup();
                    Token::Symbol(result.iter().collect::<String>().into())
                }
            }

            'f' => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.iter().collect::<String>() == "false" {
                    Token::Boolean(false)
                } else {
                    self.backup();
                    Token::Symbol(result.iter().collect::<String>().into())
                }
            }

            _ if self.get_char().is_ascii_digit() => {
                let mut result: String = String::from(self.get_char());
                let num: Regex = Regex::new("^[0-9]+(?:\\.[0-9]+)?$").unwrap();
                while num.is_match(&result) && self.get_char() != '\0'{
                    result.push(self.next());
                    if self.get_char() == '.' {
                        result.push(self.next())
                    }
                }
                if ! num.is_match(&self.get_str()) {
                    result.pop();
                    self.backup();
                }
                match result.parse() {
                    Ok(num) => Token::Number(num),
                    Err(e) => {
                        panic!(concat!("Internal Lexer Error :: Unable to parse number {} at line {},",
                        "col {}!\n Rust Error: {}"),
                               result, self.row, self.col, e);
                    }
                }
            }

            '{' => Token::OpenBrace,
            '}' => Token::CloseBrace,
            '[' => Token::OpenBracket,
            ']' => Token::CloseBracket,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '=' => Token::Equal,
            '.' => Token::Dot,
            '/' => Token::Slash,


            '\0' => if !self.discard_eof {
                Token::EoF
            } else {
                Token::Discard
            },
            _ if WHITESPACE.is_match(&self.get_str()) => if !self.discard_whitespace {
                Token::Whitespace
            } else {
                self.advance();
                return self.next_token()
            },
            _ => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.len() == 0 && self.get_char() != '\0' {
                    panic!("Unexpected Symbol {} on line {}, char {}", self.get_str(), self.row, self.col);
                } else if result.len() == 0 && self.get_char() == '\0' {
                    if !self.discard_eof {
                        Token::EoF
                    } else {
                        Token::Discard
                    }
                } else {
                    self.backup();
                    Token::Symbol(result.iter().collect::<String>().into())
                }
            }
        };
        self.advance();
        token
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    pub fn test_tokenization() {
        let text = "'This is a string' #T his is a 'comment' #aaa\n0.1231 1 0.0";
        let output = Lexer::new(text.chars().collect()).toggle_whitespace().tokenize_input();

        let output: Vec<Token> = output.to_vec();

        assert_eq!(output[0], Token::String("'This is a string'".into()));
        assert_eq!(output[1], Token::Number(0.1231.into()));
        assert_eq!(output[2], Token::Number(1.into()));
        assert_eq!(output[3], Token::Number(0.0.into()));
    }

    #[test]
    pub fn test_peek() {
        let text = "0.0 1.0";
        let mut lexer = Lexer::new(text.chars().collect()).toggle_whitespace();

        assert_eq!(lexer.peek_token(), Token::Number(0.0.into()));
        assert_eq!(lexer.next_token(), Token::Number(0.0.into()));
        assert_eq!(lexer.peek_token(), Token::Number(1.0.into()));
        assert_eq!(lexer.next_token(), Token::Number(1.0.into()));
    }

    #[test]
    pub fn test_symbol() {
        let text = "a bb test i3 gh-test";
        let mut lexer = Lexer::new(text.chars().collect()).toggle_whitespace();

        assert_eq!(lexer.next_token(), Token::Symbol(Rc::from("a")));
        assert_eq!(lexer.next_token(), Token::Symbol(Rc::from("bb")));
        assert_eq!(lexer.next_token(), Token::Symbol(Rc::from("test")));
        assert_eq!(lexer.next_token(), Token::Symbol(Rc::from("i3")));
        assert_eq!(lexer.next_token(), Token::Symbol(Rc::from("gh-test")));
    }

    #[test]
    pub fn test_eof() {
        let mut lexer = Lexer::new("".chars().collect());

        assert_eq!(lexer.peek_token(), Token::EoF);
        let output = lexer.tokenize_input();
        assert_eq!(output[0], Token::EoF);
    }

}