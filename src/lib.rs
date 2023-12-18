pub mod lex;
pub mod parser;
pub mod interpriter;
pub mod system;
mod systemdiff;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod integration {
        use std::rc::Rc;
        use crate::lex::Token;
        use crate::parser::{Expr, MapAttrExpr, NumberExpr};
        use super::*;

        #[test]
        fn test_lexer_to_parser() {
            let test_input = "system.config = { aaa = 123 }";
            let mut lexer = lex::Lexer::from_string(test_input);
            let mut parser = parser::Parser::from_token_list_smart(lexer.tokenize_input_smart());
            let parse_tree = parser.parse_input();

            let output: Rc<[Expr]> = Rc::from(vec![
                Expr::VarDecl(
                    Box::new(Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("system"))), Box::from(Expr::Symbol(Rc::from("config"))))),
                    Box::new(Expr::Map(vec![MapAttrExpr { key: Expr::Symbol(Rc::from("aaa")), value: Expr::Number(NumberExpr { num: 123.0 })}]))
                )
            ]);

            assert_eq!(output, parse_tree);
        }

        #[test]
        fn test_parser_to_interpreter() {
            let test_input = vec![
                Token::Symbol(Rc::from("system")), Token::Dot, Token::Symbol(Rc::from("config")), Token::Whitespace,
                Token::Equal, Token::Whitespace,
                Token::OpenBrace, Token::Whitespace,
                Token::Symbol(Rc::from("aaa")), Token::Whitespace,
                Token::Equal, Token::Whitespace,
                Token::Number(123.0), Token::Whitespace,
                Token::CloseBrace, Token::EoF,
            ];

            let mut parser = parser::Parser::from_token_list(Rc::from(test_input));
            let mut interpreter = interpriter::Interpreter::new(parser.parse_input());
            interpreter.eval();

            println!("{:?}", interpreter.env);
            assert!(false)
        }

        #[test]
        fn test_interpreter_full_integration() {
        }
    }
}
