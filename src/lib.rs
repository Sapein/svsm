pub mod lex;
pub mod parser;
pub mod interpreter;

pub mod system;
mod actions;

#[cfg(test)]
mod integration_tests {
    use super::*;

    use std::collections::{BTreeMap, HashMap};
    use std::rc::Rc;
    use ordered_float::OrderedFloat;
    use crate::interpreter;
    use crate::lex::Token;
    use crate::parser::{Expr, NumberExpr};
    use crate::system::{PackageRepository, Source, Service, RemoteSource};

    #[test]
    fn test_lexer_to_parser() {
        let test_input = "system.config = { aaa = 123 }";
        let mut lexer = lex::Lexer::from_string(test_input);
        let mut parser = parser::Parser::from_token_list_smart(lexer.tokenize_input_smart());
        let parse_tree = parser.parse_input();

        let output: Rc<[Expr]> = Rc::from(vec![
            Expr::VarDecl(
                Box::new(Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("system"))), Box::from(Expr::Symbol(Rc::from("config"))))),
                Box::new(Expr::Map(BTreeMap::from([(Expr::Symbol(Rc::from("aaa")), Expr::Number(NumberExpr { num: OrderedFloat::from(123.0) }))])))
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
        let mut interpreter = interpreter::Interpreter::new(parser.parse_input());
        interpreter.env.add_variable(Expr::Symbol(Rc::from("system")), Expr::Map(BTreeMap::new()));
        interpreter.eval();

        let final_variable = interpreter.env.find_variable(&Rc::from("system"));
        let expected_output = Expr::Map(BTreeMap::from([
            (Expr::Symbol(Rc::from("config")),
             Expr::Map(BTreeMap::from([
                 (Expr::Symbol(Rc::from("aaa")),
                  Expr::Number(NumberExpr::from_number(123.0)))
             ])))
        ]));
        assert_eq!(final_variable, expected_output)
    }

    #[test]
    fn test_interpreter_full_integration() {
        let test_input = "system.config = { aaa = 123 }";
        let mut lexer = lex::Lexer::from_string(test_input);
        let mut parser = parser::Parser::from_token_list_smart(lexer.tokenize_input_smart());
        let mut interpreter = interpreter::Interpreter::new(parser.parse_input());
        interpreter.env.add_variable(Expr::Symbol(Rc::from("system")), Expr::Map(BTreeMap::new()));
        interpreter.eval();

        let final_variable = interpreter.env.find_variable(&Rc::from("system"));
        let expected_output = Expr::Map(BTreeMap::from([
            (Expr::Symbol(Rc::from("config")),
             Expr::Map(BTreeMap::from([
                 (Expr::Symbol(Rc::from("aaa")),
                  Expr::Number(NumberExpr::from_number(123.0)))
             ])))
        ]));
        assert_eq!(final_variable, expected_output)
    }

    #[test]
    fn test_interpreter_to_system() {
        use parser::Expr;
        use system::System;
        use interpreter::system_converter::*;
        let mut interpreter = interpreter::Interpreter::new_vector_ast(vec![
            Expr::VarDecl(
                Box::new(Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("system"))), Box::from(Expr::Symbol(Rc::from("config"))))),
                Box::new(
                    Expr::Map(BTreeMap::from([
                        (Expr::Symbol(Rc::from("services")),
                         Expr::List(vec![Expr::Map(BTreeMap::from([
                             (Expr::Symbol(Rc::from("name")),
                              Expr::String(Rc::from("sshd"))),
                         ]))])),
                        (Expr::Symbol(Rc::from("vp_repos")),
                         Expr::Map(BTreeMap::from([
                             (Expr::Symbol(Rc::from("personal")),
                              Expr::Map(BTreeMap::from([
                                  (Expr::Symbol(Rc::from("location")),
                                   Expr::GitHubRemote {
                                       user: Rc::from("sapein"),
                                       repo: Rc::from("void-packages"),
                                       branch: None,
                                   }),
                                  (Expr::Symbol(Rc::from("branch")), Expr::String(Rc::from("personal"))),
                                  (Expr::Symbol(Rc::from("allow_restricted")), Expr::Boolean(true)),
                              ])))]))
                        ),
                ]))),
            )
        ]);
        interpreter.env.add_variable(Expr::Symbol(Rc::from("system")), Expr::Map(BTreeMap::new()));
        interpreter.eval();
        let system_config = interpreter.env
            .find_variable(&Rc::from("system"))
            .get_map_value(Expr::symbol_from_str("config"))
            .unwrap()
            .clone();
        let output = System::from_map(system_config);

        let expected = System {
            services: HashMap::from([
                (Rc::from("sshd"), Service {
                    name: Rc::from("sshd"),
                    enabled: true,
                    downed: false
                })
            ]),
            repositories: HashMap::from( [
                (Rc::from("personal"),
                 PackageRepository {
                     name: Some(Rc::from("personal")),
                     location: Source::Remote(RemoteSource::GithubRemote {
                         user: Rc::from("sapein"),
                         branch_name: Some(Rc::from("personal")),
                         repository_name:  Rc::from("void-packages"),
                     }),
                     allow_restricted: true,
                 })]),
            users: HashMap::new(),
            system_packages: Rc::from(""),
        };

        assert_eq!(output, expected);
    }
}
