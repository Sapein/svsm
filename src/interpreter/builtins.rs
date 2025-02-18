#![allow(unused)]
use std::io::Write;
use std::rc::Rc;
use crate::interpreter::{Env, eval, Interpreter};
use crate::parser::{Callable, Expr, NumberExpr};

pub(crate) fn print(args: Vec<Expr>, _env: &mut Env) -> Option<Expr> {
    // ToDo: Maybe make print a macro, since we could (theoretically) get a macro and to resolve it
    // we must *be* a macro.
    fn resolve_expr(arg: Expr, env: &mut Env) -> () {
        match arg {
            Expr::Symbol(val) => {
                print!("{} = ", val);
                resolve_expr(env.find_variable(&val), env);
            }
            Expr::String(val) => print!("{}", val),
            Expr::Boolean(bool) => print!("{}", bool),
            Expr::Number(NumberExpr { num: n})  => print!("{}", n),
            Expr::Path(path) => print!("{:?}", path.as_os_str()),
            Expr::List(list) => print!("{:?}", list),
            Expr::Map(map) => {
                print!("{{ ");
                for (key, attr) in map.clone() {
                    resolve_expr(key, env);
                    print!(" = ");
                    resolve_expr(attr, env);
                    print!("; ");
                }
                print!("}}\n");
            },

            Expr::GitHubRemote { user, repo , .. } => {
                print!("https://github.com/{}/{}", user, repo);
            },

            Expr::ListRef(sym, index) => {
                match env.find_variable_with_expr(&sym) {
                    Expr::List(list) => match list.get(index.num.into_inner() as usize) {
                        Some(expr) => resolve_expr(expr.to_owned(), env),
                        None => panic!("Index {} exceeds bounds of list {}. Bounds: {}", index.num.into_inner() as usize, sym.to_string(), list.len()),
                    }
                    _ => panic!("Can not index into a non list!")
                }
            }
            
            Expr::MapRef(sym, attr) => {
                match env.find_variable_with_expr(&sym) {
                    Expr::Map(map) => match map.get_key_value(&attr) {
                        None => panic!("Attr {} not found in map {}!", attr.to_string(), sym.to_string()),
                        Some((_, &ref val)) => resolve_expr(val.to_owned(), env),
                    }
                    _ => panic!("Attr not valid for non-map!")
                }
            }
            
            Expr::FnResult(expr) => {
                let crate::parser::FnResultExpr { function: f, args, env: call_env} = expr;
                let f = match f {
                    Callable::Builtin(b) => b,
                    Callable::Macro(_) => todo!(),
                };
                
                let result = f(args, &mut call_env.clone());
                match result {
                    None => (),
                    Some(expr) => resolve_expr(expr, env),
                }
            }
            
            Expr::FnCall(call) => print!("Call to Function {}", call.name),
            Expr::Builtin(_) => print!("Builtin Function with unknown name.", ),
            Expr::VarDecl(_, _) => panic!("Variable declaration not valid in print"),
            Expr::Macro(_) => panic!("Can not resolve macro!"),
            Expr::Action(_) => panic!("Unhandled Expression: External Action!"),
        }
    }
    for arg in args {
        resolve_expr(arg, _env)
    }
    println!();
    std::io::stdout().flush().unwrap();
    None
}

// Note: This isn't meant to be available outside of testing.
pub(crate) fn add(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    let mut total: f64 = 0.;
    for arg in args {
        match eval(arg, env, false) {
            Some(Expr::Number(expr)) => total += expr.num.into_inner(),
            _ => panic!("Add only works on numbers!"),
        }
    }
    Some(Expr::Number(NumberExpr::from_number(total)))
}

pub(crate) fn github_repo(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    fn resolve_expr(expr: &Expr, env: &mut Env) -> Option<Rc<str>> {
        match expr {
            Expr::String(str) => Some(str.clone()),
            _ if matches!(expr, Expr::Symbol(..)) => resolve_expr(&eval(expr.clone(), env, false).unwrap(), env),
            _ => None,
            _ => panic!("Unknown Type!"),
        }
    }

    if args.len() < 2 {
        panic!("Argument repo not provided to fn github_repo!");
    }

    Some(Expr::GitHubRemote {
        user: resolve_expr(args.get(0).unwrap(), env).unwrap(),
        repo: resolve_expr(args.get(1).unwrap(), env).unwrap(),
        branch: match args.get(2) {
            Some(t) => resolve_expr(t, env),
            None => None,
        }
    })
}

pub(crate) fn voidpackages_repo(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    if args.len() < 1 {
        panic!("Argument user not provided to voidpackages-repo!")
    }

    env.add_if_not_exists(Rc::from("VOID_PACKAGES_REPO_NAME"), Expr::String(Rc::from("void-packages")));

    github_repo(vec![args.get(0).unwrap().clone(), Expr::Symbol(Rc::from("VOID_PACKAGES_REPO_NAME"))], env)
}

pub(crate) fn join(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    if args.len() < 2 {
        panic!("Not enough arguments passed to join!")
    }
    
    let joiner = match &args[0] {
        Expr::String(char) => char.to_string(),
        Expr::Symbol(symbol) => match env.find_variable(symbol) {
            Expr::String(str) => str.to_string(),
            _ => panic!("First argument must be a string!"),
        },
        _ => panic!("First argument must be a string!"),
    };
    
    let list: Vec<String> = match &args[1] {
        Expr::List(list) => list.iter().map(|e| { e.to_string() }).collect(),
        _ => panic!("Second argument must be a list!"),
    };
    
    Some(Expr::String(Rc::from(list.join(joiner.as_str()))))
}

pub(crate) fn replace(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    if args.len() < 3 {
        panic!("Not enough arguments passed to replace!")
    }
    
    let original = match &args[0] {
        Expr::String(str) => str.clone(),
        Expr::Symbol(symbol) => match env.find_variable(symbol) {
            Expr::String(str) => str.clone(),
            _ => panic!("First argument must be a string!"),
        }
        _ => panic!("First argument must be a string!"),
    }.to_string();
    
    let replacement = match &args[1] {
        Expr::String(str) => str.clone(),
        Expr::Symbol(symbol) => match env.find_variable(symbol) {
            Expr::String(str) => str.clone(),
            _ => panic!("First argument must be a string!"),
        }
        _ => panic!("First argument must be a string!"),
    };
    
    let string = match &args[2] {
        Expr::String(str) => str.clone(),
        Expr::Symbol(symbol) => match env.find_variable(symbol) {
            Expr::String(str) => str.clone(),
            _ => panic!("First argument must be a string!"),
        }
        _ => panic!("First argument must be a string!"),
    };
    
    let replaced = string.replace(&original.to_string(), &replacement);
    
    Some(Expr::String(Rc::from(string.replace(&original, &replacement))))
}

pub(crate) fn use_file(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    todo_fn(args, env)
}

pub(crate) fn remove(args: Vec<Expr>, env: &mut Env) -> Option<Expr> {
    todo_fn(args, env)
}

pub(crate) fn todo_fn(args: Vec<Expr>, _env: &mut Env) -> Option<Expr> {
    todo!()
}

pub(crate) fn todo_macro(args: Vec<Expr>, interpreter: &mut Interpreter) -> Option<Expr> {
    todo!()
}