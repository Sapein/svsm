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

    row: usize,
    col: usize
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    String(Rc<[char]>),
    Path(Rc<[char]>),
    Boolean(bool),
    Number(Rc<[char]>),
    Symbol(Rc<[char]>),
    Semicolon,
    Comma,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    ClosedBracket,
    Equal,
    Dot,

    Discard,
}


impl Lexer {
    fn new(input: Vec<char>) -> Self {
        Self {
            input,
            pos: 0,

            col: 1,
            row: 1,
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
        self.pos += 1;
        self.col += 1;
        if self.get_char() == '\n' {
            self.row += 1;
            self.col = 1;
        }
        self
    }

    fn next(&mut self) -> char {
        self.advance();
        self.get_char()
    }

    fn collect_to(&mut self, pattern: &Regex) -> Vec<char> {
        let mut tokens: Vec<char> = vec!(self.get_char());
        self.advance();
        while !pattern.is_match(&self.get_str()) && self.peek_next() != '\0'{
            tokens.push(self.get_char());
            self.advance();
        }
        tokens.push(self.get_char());
        tokens
    }

    fn advance_until(&mut self, pattern: &Regex) -> &Self {
        while !pattern.is_match(&self.peek_next_str()) && self.peek_next() != '\0' {
            self.advance();
        }
        self
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
        let mut tokens: Vec<Token> = vec!();
        while self.pos <= self.input.len() {
            let token = self.next_token();
            match token {
                Token::Discard => (),
                _ => tokens.push(token)
            }
        }
        tokens.into()
    }

    pub fn next_token(&mut self) -> Token {
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
                Token::String(result.into())
            },
            '"' => {
                let (row, col) = (self.row, self.col);
                let result = self.collect_to(&Regex::new("\"").unwrap());
                if result.last().unwrap() != &'"' {
                    panic!("String opened on line {}, char {} not closed until end of file!\n String: {}", row, col, result.iter().collect::<String>());
                }
                Token::String(result.into())
            }
            '/' => Token::Path(self.collect_until(&BREAKING).into()),
            '.' if self.peek_next() == '/' => Token::Path(self.collect_until(&BREAKING).into()),

            't' => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.iter().collect::<String>() == "true" {
                    Token::Boolean(true)
                } else {
                    Token::Symbol(result.into())
                }
            }

            'f' => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.iter().collect::<String>() == "false" {
                    Token::Boolean(false)
                } else {
                    Token::Symbol(result.into())
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
                Token::Number(result.chars().collect())
            }

            '{' => Token::OpenBrace,
            '}' => Token::CloseBrace,
            '[' => Token::OpenBracket,
            ']' => Token::ClosedBracket,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '=' => Token::Equal,
            '.' => Token::Dot,


            _ if WHITESPACE.is_match(&self.get_str()) => Token::Discard,
            _ => {
                let result = self.collect_while(&VALID_SYMBOL);
                if result.len() == 0 {
                    panic!("Unexpected Symbol {} on line {}, char {}", self.get_str(), self.row, self.col);
                }
                Token::Symbol(self.collect_while(&VALID_SYMBOL).into())
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