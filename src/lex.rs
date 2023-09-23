use std::rc::Rc;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WHITESPACE: Regex = Regex::new("\\s").unwrap();
    static ref BREAKING: Regex = Regex::new("\\s|\\{|\\}|;|,|\\[|\\]|=").unwrap();
    static ref VALID_SYMBOL: Regex = Regex::new("[A-Za-z_]+[A-Za-z_0-9\\-]*").unwrap();
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    String(Vec<char>),
    Path(Vec<char>),
    Boolean(bool),
    Number(Vec<char>),
    Symbol(Vec<char>),
    Semicolon,
    Comma,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    ClosedBracket,
    Equal,
    Dot,
}


impl Lexer {
    fn new(input: Vec<char>) -> Self {
        Self {
            input,
            pos: 0,
        }
    }

    fn peek_next(&self) -> char {
        if self.pos + 1 >= self.input.len() {
            return '\0';
        }
        self.input[self.pos + 1]
    }

    fn peek_next_str(&self) -> String {
        String::from(self.peek_next())
    }

    fn get_char(&self) -> char {
        if self.pos >= self.input.len() {
            return '\0';
        }
        self.input[self.pos]
    }

    fn get_str(&self) -> String {
       String::from(self.get_char())
    }

    fn advance(&mut self) -> &Self {
        if self.pos + 1 <= self.input.len() {
            self.pos += 1;
        }
        self
    }

    fn next(&mut self) -> char {
        self.advance();
        self.get_char()
    }

    fn advance_until(&mut self, pattern: &Regex) -> &Self {
        while !pattern.is_match(&self.get_str()) && self.get_char() != '\0' {
            self.advance();
        }
        self
    }

    fn collect_to(&mut self, pattern: &Regex) -> Vec<char> {
        let mut tokens: Vec<char> = vec!(self.get_char());
        self.advance();
        while !pattern.is_match(&self.get_str()){
            tokens.push(self.get_char());
            self.advance();
        }
        tokens.push(self.get_char());
        tokens
    }

    fn collect_until(&mut self, pattern: &Regex) -> Vec<char> {
        let mut token: Vec<char> = vec!(self.get_char());
        while !pattern.is_match(&self.peek_next_str()) && self.peek_next() != '\0' {
            token.push(self.next());
        }
        token
    }

    fn collect_while(&mut self, pattern: &Regex) -> Vec<char> {
        let mut token: Vec<char> = vec!();
        while pattern.is_match(&self.get_str()) && self.get_char() != '\0' {
            token.push(self.get_char());
            self.advance();
        }
        token
    }

    pub fn tokenize_input(&mut self) -> Rc<[Token]> {
        let mut results: Vec<Token> = vec!();

        while self.pos < self.input.len() {
            match self.get_char() {
                '#' => {self.advance_until(&Regex::new("\\n").unwrap());},
                '\'' => results.push({Token::String(self.collect_to(&Regex::new("'").unwrap()))}),
                '"' => results.push({Token::String(self.collect_to(&Regex::new("\"").unwrap()))}),
                '/' => results.push(Token::Path(self.collect_until(&BREAKING))),
                '.' if self.peek_next() == '/' => results.push(Token::Path(self.collect_until(&BREAKING))),

                't' => {
                    let result = self.collect_while(&VALID_SYMBOL);
                    if result.iter().collect::<String>() == "true" {
                        results.push(Token::Boolean(true));
                    }
                }

                'f' => {
                    let result = self.collect_while(&VALID_SYMBOL);
                    if result.iter().collect::<String>() == "false" {
                        results.push(Token::Boolean(false));
                    }
                }

                _ if self.get_char().is_ascii_digit() => {
                    let mut result: String = String::from(self.get_char());
                    while Regex::new("^[0-9]+(?:\\.[0-9]+)?$").unwrap().is_match(&result) && self.get_char() != '\0'{
                        result.push(self.next());
                        if self.get_char() == '.' {
                            result.push(self.next())
                        }
                    }
                    if WHITESPACE.is_match(&self.get_str()) || self.get_char() == '\0' { result.pop(); }
                    results.push(Token::Number(result.chars().collect()));
                }

                '{' => results.push(Token::OpenBrace),
                '}' => results.push(Token::CloseBrace),
                '[' => results.push(Token::OpenBracket),
                ']' => results.push(Token::ClosedBracket),
                ';' => results.push(Token::Semicolon),
                ',' => results.push(Token::Comma),
                '=' => results.push(Token::Equal),
                '.' => results.push(Token::Dot),


                _ if WHITESPACE.is_match(&self.get_str()) => (),
                _ => {
                    let result = self.collect_while(&VALID_SYMBOL);
                    if result.len() == 0 {
                        panic!("Unknown symbol: {}", self.get_str());
                    }
                    results.push(Token::Symbol(self.collect_while(&VALID_SYMBOL)));
                }
            }
            self.advance();
        }
        results.into()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    pub fn test_parse() {
        let text = "'This is a string' /this/is/a/path.txt ./path.txt #T his is a 'comment' #aaa\n0.1231 1 0.0";
        let output = Lexer::new(text.chars().collect()).tokenize_input();

        let output: Vec<Token> = output.to_vec();

        assert_eq!(output[0], Token::String("'This is a string'".chars().collect()));
        assert_eq!(output[1], Token::Path("/this/is/a/path.txt".chars().collect()));
        assert_eq!(output[2], Token::Path("./path.txt".chars().collect()));
        assert_eq!(output[3], Token::Number("0.1231".chars().collect()));
        assert_eq!(output[4], Token::Number("1".chars().collect()));
        assert_eq!(output[5], Token::Number("0.0".chars().collect()));
    }

}