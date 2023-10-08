use std::path::PathBuf;
use std::rc::Rc;
use svsm::parser::{Parser, Expr, NumberExpr, MapAttrExpr, ExprFnCall};
use svsm::lex::{Lexer, Token};

#[test]
fn simple_output_no_whitespace() {
    let input_str = "a.b\nc[1]\nd[1.0]\n[1, 2,]\n{ e = 1; }\n";
    let lexer = Lexer::from_string(input_str);
    let tokenizer_expected  = Rc::from(vec![
        Token::Symbol(Rc::from("a")), Token::Dot, Token::Symbol(Rc::from("b")),
        Token::Symbol(Rc::from("c")), Token::OpenBracket, Token::Number(1.0), Token::CloseBracket,
        Token::Symbol(Rc::from("d")), Token::OpenBracket, Token::Number(1.0), Token::CloseBracket,
        Token::OpenBracket, Token::Number(1.0), Token::Comma,  Token::Number(2.0), Token::Comma, Token::CloseBracket,
        Token::OpenBrace,  Token::Symbol(Rc::from("e")),  Token::Equal,  Token::Number(1.0), Token::Semicolon,  Token::CloseBrace,
        Token::EoF,
    ]);

    let lexer_output = lexer.toggle_whitespace().tokenize_input();
    assert_eq!(tokenizer_expected, lexer_output);


    let mut parser = Parser::from_token_list(lexer_output);
    let parser_expected = Rc::from(vec![
        Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("a"))), Rc::from(Expr::Symbol(Rc::from("b")))),
        Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("c"))), NumberExpr::from_number(1.0)),
        Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("d"))), NumberExpr::from_number(1.0)),
        Expr::List(Vec::from([Expr::Number(NumberExpr::from_number(1.0)), Expr::Number(NumberExpr::from_number(2.0))])),
        Expr::Map(Vec::from([MapAttrExpr::new(Expr::Symbol(Rc::from("e")), Expr::Number(NumberExpr::from_number(1.0)))])),
    ]);

    let output = parser.parse_input();
    assert_eq!(parser_expected, output);
}

#[test]
fn complex_output_no_whitespace() {
    let input_str = "system.config = {
        users = [
            {
                username = 'sapeint';
                homedir = {
                    subdirs = [ ./library, ./games/launchers,];
                };
                dotfiles = gh-r 'sapein' 'dotfiles'; # This will be used, but partially overridden
                packages = [
                    i3status, i3lock, dmenu, firefox,
                    i3 {
                        config = use_file ./i3/config (gh-r 'sapein' 'dotfiles');
                    },
                    discord { repository = personal; },
                ];
            }
            ,];

        services = [{name = sshd;},];
    }";

    let mut lexer = Lexer::from_string(input_str).toggle_whitespace();
    let tokenizer_expected  = Rc::from(vec![
        Token::Symbol(Rc::from("system")), Token::Dot, Token::Symbol(Rc::from("config")), Token::Equal, Token::OpenBrace,
        Token::Symbol(Rc::from("users")), Token::Equal, Token::OpenBracket,
        Token::OpenBrace,
        Token::Symbol(Rc::from("username")), Token::Equal, Token::String(Rc::from("'sapeint'")), Token::Semicolon,
        Token::Symbol(Rc::from("homedir")), Token::Equal, Token::OpenBrace,
        Token::Symbol(Rc::from("subdirs")), Token::Equal, Token::OpenBracket, Token::Dot, Token::Slash, Token::Symbol(Rc::from("library")), Token::Comma, Token::Dot, Token::Slash, Token::Symbol(Rc::from("games")), Token::Slash, Token::Symbol(Rc::from("launchers")), Token::Comma, Token::CloseBracket, Token::Semicolon,
        Token::CloseBrace, Token::Semicolon,
        Token::Symbol(Rc::from("dotfiles")), Token::Equal, Token::Symbol(Rc::from("gh-r")), Token::String(Rc::from("'sapein'")), Token::String(Rc::from("'dotfiles'")), Token::Semicolon,
        Token::Symbol(Rc::from("packages")), Token::Equal, Token::OpenBracket,
        Token::Symbol(Rc::from("i3status")), Token::Comma, Token::Symbol(Rc::from("i3lock")), Token::Comma, Token::Symbol(Rc::from("dmenu")), Token::Comma, Token::Symbol(Rc::from("firefox")), Token::Comma,
        Token::Symbol(Rc::from("i3")), Token::OpenBrace,
        Token::Symbol(Rc::from("config")), Token::Equal, Token::Symbol(Rc::from("use_file")), Token::Dot, Token::Slash, Token::Symbol(Rc::from("i3")), Token::Slash, Token::Symbol(Rc::from("config")), Token::OpenParen, Token::Symbol(Rc::from("gh-r")), Token::String(Rc::from("'sapein'")), Token::String(Rc::from("'dotfiles'")), Token::CloseParen, Token::Semicolon,
        Token::CloseBrace, Token::Comma,
        Token::Symbol(Rc::from("discord")), Token::OpenBrace, Token::Symbol(Rc::from("repository")), Token::Equal, Token::Symbol(Rc::from("personal")), Token::Semicolon, Token::CloseBrace, Token::Comma,
        Token::CloseBracket, Token::Semicolon,
        Token::CloseBrace,
        Token::Comma, Token::CloseBracket, Token::Semicolon,
        Token::Symbol(Rc::from("services")), Token::Equal, Token::OpenBracket, Token::OpenBrace, Token::Symbol(Rc::from("name")), Token::Equal, Token::Symbol(Rc::from("sshd")), Token::Semicolon, Token::CloseBrace, Token::Comma, Token::CloseBracket, Token::Semicolon,
        Token::CloseBrace,
        Token::EoF,
    ]);

    let lexer_output = lexer.tokenize_input();
    assert_eq!(tokenizer_expected, lexer_output);

    let mut parser = Parser::from_token_list(lexer_output);
    let parser_expected = Rc::from(vec![
        Expr::VarDecl(Rc::from(Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("system"))), Rc::from(Expr::Symbol(Rc::from("config"))))),
        Rc::from(Expr::Map(vec![
            MapAttrExpr {
                key: Expr::Symbol(Rc::from("users")),
                value: Expr::List(vec![
                    Expr::Map(vec![
                        MapAttrExpr {
                            key: Expr::Symbol(Rc::from("username")),
                            value: Expr::String(Rc::from("'sapeint'")),
                        },
                        MapAttrExpr {
                            key: Expr::Symbol(Rc::from("homedir")),
                            value: Expr::Map(vec![
                                MapAttrExpr {
                                    key: Expr::Symbol(Rc::from("subdirs")),
                                    value: Expr::List(vec![
                                        Expr::Path(PathBuf::from("./library")),
                                        Expr::Path(PathBuf::from("./games/launchers")),
                                    ])
                                }
                            ])
                        },
                        MapAttrExpr {
                            key: Expr::Symbol(Rc::from("dotfiles")),
                            value: Expr::FnCall(
                                ExprFnCall {
                                    name: Rc::from("gh-r"),
                                    args: vec![
                                        Expr::String(Rc::from("'sapein'")),
                                        Expr::String(Rc::from("'dotfiles'")),
                                    ]
                                }
                            )
                        },
                        MapAttrExpr {
                            key: Expr::Symbol(Rc::from("packages")),
                            value: Expr::List(vec![
                                Expr::Symbol(Rc::from("i3status")),
                                Expr::Symbol(Rc::from("i3lock")),
                                Expr::Symbol(Rc::from("dmenu")),
                                Expr::Symbol(Rc::from("firefox")),
                                Expr::FnCall(ExprFnCall {
                                    name: Rc::from("i3"),
                                    args: vec![
                                        Expr::Map(vec![
                                            MapAttrExpr {
                                                key: Expr::Symbol(Rc::from("config")),
                                                value: Expr::FnCall(ExprFnCall {
                                                    name: Rc::from("use_file"),
                                                    args: vec![
                                                        Expr::Path(PathBuf::from("./i3/config")),
                                                        Expr::FnCall(ExprFnCall {
                                                            name: Rc::from("gh-r"),
                                                            args: vec![
                                                                Expr::String(Rc::from("'sapein'")),
                                                                Expr::String(Rc::from("'dotfiles'")),
                                                            ]
                                                        })
                                                    ]
                                                }),
                                            }
                                        ])]
                                }),
                                Expr::FnCall(ExprFnCall {
                                    name: Rc::from("discord"),
                                    args: vec![
                                        Expr::Map(vec![
                                            MapAttrExpr {
                                                key: Expr::Symbol(Rc::from("repository")),
                                                value: Expr::Symbol(Rc::from("personal"))
                                            }])]
                                })
                            ]),
                        },
                    ])
                ]),
            },
            MapAttrExpr {
                key: Expr::Symbol(Rc::from("services")),
                value: Expr::List(vec![
                    Expr::Map(vec![
                        MapAttrExpr {
                            key: Expr::Symbol(Rc::from("name")),
                            value: Expr::Symbol(Rc::from("sshd")),
                        }
                    ])
                ])
            }
        ]))
        ),
    ]);

    let output = parser.parse_input();
    println!("{:#?}", output);
    assert_eq!(parser_expected, output);
}

#[test]
fn simple_output() {
    let input_str = "a.b\nc[1]\nd[1.0]\n[1, 2,]\n{ e = 1; }\n";
    let mut lexer = Lexer::from_string(input_str);
    let tokenizer_expected  = Rc::from(vec![
        Token::Symbol(Rc::from("a")), Token::Dot, Token::Symbol(Rc::from("b")), Token::Whitespace,
        Token::Symbol(Rc::from("c")), Token::OpenBracket, Token::Number(1.0), Token::CloseBracket, Token::Whitespace,
        Token::Symbol(Rc::from("d")), Token::OpenBracket, Token::Number(1.0), Token::CloseBracket, Token::Whitespace,
        Token::OpenBracket, Token::Number(1.0), Token::Comma,  Token::Whitespace, Token::Number(2.0), Token::Comma, Token::CloseBracket, Token::Whitespace,
        Token::OpenBrace,  Token::Whitespace, Token::Symbol(Rc::from("e")),  Token::Whitespace, Token::Equal,  Token::Whitespace, Token::Number(1.0), Token::Semicolon,  Token::Whitespace, Token::CloseBrace,
        Token::Whitespace, Token::EoF,
    ]);

    let lexer_output = lexer.tokenize_input();
    assert_eq!(tokenizer_expected, lexer_output);


    let mut parser = Parser::from_token_list(lexer_output);
    let parser_expected = Rc::from(vec![
        Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("a"))), Rc::from(Expr::Symbol(Rc::from("b")))),
        Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("c"))), NumberExpr::from_number(1.0)),
        Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("d"))), NumberExpr::from_number(1.0)),
        Expr::List(Vec::from([Expr::Number(NumberExpr::from_number(1.0)), Expr::Number(NumberExpr::from_number(2.0))])),
        Expr::Map(Vec::from([MapAttrExpr::new(Expr::Symbol(Rc::from("e")), Expr::Number(NumberExpr::from_number(1.0)))])),
    ]);

    let output = parser.parse_input();
    assert_eq!(parser_expected, output);
}

#[test]
fn complex_output() {
    let input_str = "system.config = {
        users = [
            {
                username = 'sapeint';
                homedir = {
                    subdirs = [ ./library, ./games/launchers,];
                };
                dotfiles = gh-r 'sapein' 'dotfiles'; # This will be used, but partially overridden
                packages = [
                    i3status, i3lock, dmenu, firefox,
                    i3 {
                        config = use_file ./i3/config (gh-r 'sapein' 'dotfiles');
                    },
                    discord { repository = personal; },
                ];
            }
            ,];

        services = [{name = sshd;},];
    }";

    let mut lexer = Lexer::from_string(input_str);

    let lexer_output = lexer.tokenize_input_smart();

    let mut parser = Parser::from_token_list_smart(lexer_output);
    let parser_expected = Rc::from(vec![
        Expr::VarDecl(Rc::from(Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("system"))), Rc::from(Expr::Symbol(Rc::from("config"))))),
                      Rc::from(Expr::Map(vec![
                          MapAttrExpr {
                              key: Expr::Symbol(Rc::from("users")),
                              value: Expr::List(vec![
                                  Expr::Map(vec![
                                      MapAttrExpr {
                                          key: Expr::Symbol(Rc::from("username")),
                                          value: Expr::String(Rc::from("'sapeint'")),
                                      },
                                      MapAttrExpr {
                                          key: Expr::Symbol(Rc::from("homedir")),
                                          value: Expr::Map(vec![
                                              MapAttrExpr {
                                                  key: Expr::Symbol(Rc::from("subdirs")),
                                                  value: Expr::List(vec![
                                                      Expr::Path(PathBuf::from("./library")),
                                                      Expr::Path(PathBuf::from("./games/launchers")),
                                                  ])
                                              }
                                          ])
                                      },
                                      MapAttrExpr {
                                          key: Expr::Symbol(Rc::from("dotfiles")),
                                          value: Expr::FnCall(
                                              ExprFnCall {
                                                  name: Rc::from("gh-r"),
                                                  args: vec![
                                                      Expr::String(Rc::from("'sapein'")),
                                                      Expr::String(Rc::from("'dotfiles'")),
                                                  ]
                                              }
                                          )
                                      },
                                      MapAttrExpr {
                                          key: Expr::Symbol(Rc::from("packages")),
                                          value: Expr::List(vec![
                                              Expr::Symbol(Rc::from("i3status")),
                                              Expr::Symbol(Rc::from("i3lock")),
                                              Expr::Symbol(Rc::from("dmenu")),
                                              Expr::Symbol(Rc::from("firefox")),
                                              Expr::FnCall(ExprFnCall {
                                                  name: Rc::from("i3"),
                                                  args: vec![
                                                      Expr::Map(vec![
                                                          MapAttrExpr {
                                                              key: Expr::Symbol(Rc::from("config")),
                                                              value: Expr::FnCall(ExprFnCall {
                                                                  name: Rc::from("use_file"),
                                                                  args: vec![
                                                                      Expr::Path(PathBuf::from("./i3/config")),
                                                                      Expr::FnCall(ExprFnCall {
                                                                          name: Rc::from("gh-r"),
                                                                          args: vec![
                                                                              Expr::String(Rc::from("'sapein'")),
                                                                              Expr::String(Rc::from("'dotfiles'")),
                                                                          ]
                                                                      })
                                                                  ]
                                                              }),
                                                          }
                                                      ])]
                                              }),
                                              Expr::FnCall(ExprFnCall {
                                                  name: Rc::from("discord"),
                                                  args: vec![
                                                      Expr::Map(vec![
                                                          MapAttrExpr {
                                                              key: Expr::Symbol(Rc::from("repository")),
                                                              value: Expr::Symbol(Rc::from("personal"))
                                                          }])]
                                              })
                                          ]),
                                      },
                                  ])
                              ]),
                          },
                          MapAttrExpr {
                              key: Expr::Symbol(Rc::from("services")),
                              value: Expr::List(vec![
                                  Expr::Map(vec![
                                      MapAttrExpr {
                                          key: Expr::Symbol(Rc::from("name")),
                                          value: Expr::Symbol(Rc::from("sshd")),
                                      }
                                  ])
                              ])
                          }
                      ]))
        ),
    ]);

    let output = parser.parse_input();
    println!("{:#?}", output);
    assert_eq!(parser_expected, output);
}
