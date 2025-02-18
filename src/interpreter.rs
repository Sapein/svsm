//! VSL Interpreter for SVSM.
//!
//! This provides a relatively basic parser for SVSM to use
//! to understand it's language (VSL).
//!
//! # Examples
//! ```
//! use std::rc::Rc;
//! let mut interpreter = svsm::interpreter::Interpreter::new_vector_ast(vec![svsm::parser::Expr::String(Rc::from("A string"))]);
//! println!("Output: {:?}" , interpreter.eval());
//! ```
use std::collections::BTreeMap;
use std::rc::Rc;
use crate::actions::Action;
use crate::parser::{Callable, Expr, FnResultExpr};

mod builtins;
pub mod system_converter;

pub struct Interpreter {
    input: InterpreterInput,
    pos: usize,

    // this exists mostly to allow us to disable lazy eval for automated testing purposes. 
    // It changes very little else. // this should, broadly, never be actually set for non-testing
    // code.
    // 
    // The only change with this is that FnResults are immediately evaluated.
    pub(crate) disable_lazy: bool,
    pub(crate) actions: Vec<Action>,

    pub(crate) env: Box<Env>,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Env {
    variables: BTreeMap<Rc<str>, Expr>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            variables: BTreeMap::new(),
            parent: None
        }
    }
    

    pub fn add_parent(self, parent: &Self) -> Self {
        Env {
            parent: Some(Rc::from(parent.clone())),
            ..self
        }
    }

    pub fn add_if_not_exists_with_expr(&mut self, name: Expr, value: Expr) -> &Self {
        match name.clone() {
            Expr::Symbol(sym) if self.variables.contains_key(&sym) => self.add_variable(name, value),
            Expr::Symbol(_) => self,
            _ => panic!("Variable must be a symbol!"),
        }
    }

    pub fn add_if_not_exists(&mut self, name: Rc<str>, value: Expr) -> &Self {
        if !self.variables.contains_key(&name) {
            self.add_variable(Expr::Symbol(name), value);
        }
        self
    }

    pub fn add_variable(&mut self, name: Expr, value: Expr) -> &Self {
        match name {
            Expr::Symbol(symbol) => {
                self.variables.insert(symbol, value);
            },
            Expr::MapRef(name, attr) => {
                match &*name {
                    Expr::Symbol(name) => if let Some(map) = self.get_variable(&name) {
                        match map {
                            Expr::Map(mut map) => {
                                map.insert(*attr, value);
                                self.add_variable(Expr::Symbol(name.clone()), Expr::Map(map));
                            },
                            _ => panic!("You can't do attr access on a non-map type!"),
                        }
                    } else {
                        panic!("Map {} does not exist in env: {:?}!", name, self.variables);
                    },
                    _ => panic!("Variable must be symbol!"),
                }
            }
            _ => panic!("Variable must be a symbol!"),
        };
        self
    }

    pub fn find_variable_with_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Symbol(sym) => self.find_variable(sym),
            _ => panic!("Variable must be a symbol!"),
        }
    }

    fn get_variable(&self, name:  &Rc<str>) -> Option<Expr> {
        match self.variables.get_key_value(name) {
            Some((_, v)) => Some(v.clone()),
            None => match &self.parent {
                Some(p) => Some(p.find_variable(name)),
                None => None,
            }
        }
    }

    pub fn find_variable(&self, name: &Rc<str>) -> Expr {
        match self.get_variable(name) {
            Some(t) => t,
            None => panic!("Variable with name {} not found!", name),
        }
    }
}

pub enum InterpreterInput {
    VecAst(Vec<Expr>),
    ArrAst(Rc<[Expr]>),
}

impl Interpreter {
    pub fn new_vector_ast(input: Vec<Expr>) -> Self{
       Self {
           input: InterpreterInput::VecAst(input),
           pos: 0,

           disable_lazy: false,

           actions: vec![],
           env: Box::from(Env::new()),
       }
    }
    
    pub fn create_standard_env(mut self) -> Self {
        self.env.add_variable(Expr::Symbol(Rc::from("system")), Expr::Map(BTreeMap::new()));
        self.env.add_variable(Expr::Symbol(Rc::from("print")), Expr::Builtin(builtins::print));
        
        self.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        self.env.add_variable(Expr::Symbol(Rc::from("vp-r")), Expr::Builtin(builtins::voidpackages_repo));
        
        self.env.add_variable(Expr::Symbol(Rc::from("github-repo")), Expr::Builtin(builtins::github_repo));
        self.env.add_variable(Expr::Symbol(Rc::from("voidpackages-repo")), Expr::Builtin(builtins::voidpackages_repo));
        
        self.env.add_variable(Expr::Symbol(Rc::from("home")), Expr::Builtin(builtins::todo_fn));
        self.env.add_variable(Expr::Symbol(Rc::from("replace")), Expr::Builtin(builtins::todo_fn));
        self.env.add_variable(Expr::Symbol(Rc::from("use_file")), Expr::Builtin(builtins::todo_fn));
        self.env.add_variable(Expr::Symbol(Rc::from("join")), Expr::Builtin(builtins::todo_fn));
        self
    }
    
    pub fn new(input: Rc<[Expr]>) -> Self {
        Self {
            input: InterpreterInput::ArrAst(input),
            pos: 0,

            disable_lazy: false,

            actions: vec![],
            env: Box::from(Env::new()),
        }
    }

    pub fn advance(&mut self) -> &mut Self {
        match &self.input {
            InterpreterInput::VecAst(input) => {
                if self.pos + 1 < input.len() {
                    self.pos += 1;
                }
            },
            InterpreterInput::ArrAst(input) => {
                if self.pos + 1 < input.len() {
                    self.pos += 1;
                }
            },
        }
        self
    }

    fn get_input(&self) -> Expr {
        match &self.input {
            InterpreterInput::VecAst(vec) => vec[self.pos].clone(),
            InterpreterInput::ArrAst(arr) => arr[self.pos].clone(),
        }
    }

    pub fn eval(&mut self) -> Option<Expr> {
        eval(self.get_input(), &mut self.env, self.disable_lazy)
    }

    pub fn eval_input(&mut self, input: Expr) -> Option<Expr> {
        eval(input, &mut self.env, self.disable_lazy)
    }
}


pub fn eval(input: Expr, env: &mut Env, disable_lazy: bool) -> Option<Expr> {
    match input {
        Expr::VarDecl(name, value) => {
            env.add_variable(*name, *value.clone());

            Some(*value)
        },

        Expr::Symbol(sym) => {
            Some(env.find_variable(&sym))
        },

        Expr::ListRef(sym, index) => {
            let value = env.find_variable_with_expr(&sym);
            match value {
                Expr::List(list) => match list.get(index.num.into_inner() as usize){
                    Some(t) => Some(t.clone()),
                    None => panic!("Invalid index"),
                },
                _ => panic!("Unable to list access into a non-list!"),
            }
        },

        Expr::MapRef(sym, attr) => {
            let value = env.find_variable_with_expr(&sym);
            match value {
                Expr::Map(map) => {
                    match map.get_key_value(&attr) {
                        None => panic!("Map Attr not found!"),
                        Some((_, &ref t)) => Some(t.clone()),
                    }
                }
                _ => panic!("Unable to list access into a non-list!"),
            }
        },

        Expr::FnResult(expr) => {
            let FnResultExpr { function: f, args, env } = expr;
            match f {
                Callable::Builtin(f) => f(args, &mut env.clone()),
                Callable::Macro(_) => todo!()
            }
        },
        
        Expr::FnCall(fncall) => {
            let function = env.find_variable(&fncall.name);
            match function {
                Expr::Builtin(cb) => {
                    if disable_lazy {
                        eval(Expr::FnResult(FnResultExpr {
                            function: Callable::Builtin(cb),
                            args: fncall.args,
                            env: env.clone(),
                        }), env, disable_lazy)
                    } else {
                        Some(Expr::FnResult(FnResultExpr {
                            function: Callable::Builtin(cb),
                            args: fncall.args,
                            env: env.clone(),
                        }))
                    }
                },
                Expr::Macro(_) => {
                    todo!("Macros not implemented")
                }
                Expr::FnResult(_) => panic!("FnResult attempted to be called!"),
                _ => panic!("Attempted to call a non-function!"),
            }
        },

        _ => Some(input)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;
    use ordered_float::OrderedFloat;
    use crate::parser::{ExprFnCall, NumberExpr};
    use super::*;

    #[test]
    pub fn test_evaluation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::Boolean(true)]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true))
    }

    #[test]
    pub fn test_evaluation_with_advance() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::Boolean(true)]);

        interpriter.advance();
        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true))
    }

    #[test]
    pub fn test_evaluation_with_advance_multiple() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::Boolean(true), Expr::String(Rc::from("This is a string"))]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
        assert_eq!(interpriter.advance().eval().unwrap(), Expr::String(Rc::from("This is a string")));
    }

    #[test]
    pub fn test_vardecl_evaulation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::VarDecl(Box::from(Expr::Symbol(Rc::from("test"))), Box::from(Expr::Boolean(true)))]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
        assert_eq!(interpriter.env.variables[&Rc::from("test")], Expr::Boolean(true));
    }

    #[test]
    pub fn test_symbol_evaulation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::Symbol(Rc::from("test"))]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::Number(NumberExpr::from_number(1.0)));

        assert_eq!(interpriter.eval().unwrap(), Expr::Number(NumberExpr::from_number(1.0)));
    }

    #[test]
    pub fn test_list_index_evaulation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("test"))), NumberExpr { num: OrderedFloat::from(0.0) })]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::List(vec![Expr::Boolean(true)]));

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
    }

    #[test]
    pub fn test_map_access_evaluation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("test"))), Box::from(Expr::Symbol(Rc::from("test"))))]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::Map(BTreeMap::from([(Expr::Symbol(Rc::from("test")), Expr::Path(PathBuf::from("/home")))])));

        assert_eq!(interpriter.eval().unwrap(), Expr::Path(PathBuf::from("/home")));
    }

    #[test]
    pub fn test_fncall_evaluation() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("add"), args: vec![Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) }), Expr::Number(NumberExpr { num: OrderedFloat::from(1.0) })]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("add")), Expr::Builtin(builtins::add));

        assert_eq!(interpriter.eval().unwrap(), Expr::Number(NumberExpr { num: OrderedFloat::from(1.0 + 1.0) }));
    }

    #[test]
    pub fn test_ghr_builtin_simple() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::String(Rc::from("test")), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2"), branch: None} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::Symbol(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test2")), Expr::String(Rc::from("test2")));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2"), branch: None} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols_nested() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::Symbol(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test2")), Expr::Symbol(Rc::from("test3")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test3")), Expr::String(Rc::from("test2")));


        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2"), branch: None} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols_mixed() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2"), branch: None} );
    }

    #[test]
    #[should_panic]
    pub fn test_ghr_builtin_bad_args() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Boolean(true), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));

       interpriter.eval();
    }
    
    #[test]
    pub fn test_join() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("join"), args: vec![Expr::String(Rc::from(",")), Expr::List(vec![Expr::String(Rc::from("alpha")), Expr::String(Rc::from("beta")), Expr::Number(NumberExpr { num: 1.into()})])]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("join")), Expr::Builtin(builtins::join));

        assert_eq!(interpriter.eval().unwrap(), Expr::String(Rc::from("alpha,beta,1")) );
    }
    
    #[test]
    pub fn test_join_sym() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("join"), args: vec![Expr::Symbol(Rc::from("test")), Expr::List(vec![Expr::String(Rc::from("alpha")), Expr::String(Rc::from("beta"))])]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("join")), Expr::Builtin(builtins::join));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from(",")));

        assert_eq!(interpriter.eval().unwrap(), Expr::String(Rc::from("alpha,beta")) );
    }
    
    #[test]
    #[should_panic]
    pub fn test_join_bad_arg1() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("join"), args: vec![Expr::Number(NumberExpr { num:1.into(), }), Expr::List(vec![Expr::String(Rc::from("alpha")), Expr::String(Rc::from("beta"))])]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("join")), Expr::Builtin(builtins::join));

        interpriter.eval();
    }
    
    #[test]
    #[should_panic]
    pub fn test_join_bad_args2() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("join"), args: vec![Expr::String(Rc::from(",")), Expr::Number(NumberExpr { num:1.into(), })]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("join")), Expr::Builtin(builtins::join));

        interpriter.eval();
    }
    
    #[test]
    pub fn test_replace() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("replace"), args: vec![Expr::String(Rc::from(".")), Expr::String(Rc::from(",")), Expr::String(Rc::from("a.b"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("replace")), Expr::Builtin(builtins::replace));

        assert_eq!(interpriter.eval().unwrap(), Expr::String(Rc::from("a,b")) );
    }
    
    #[test]
    #[should_panic]
    pub fn test_replace_bad_arg1() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("replace"), args: vec![Expr::Boolean(true), Expr::String(Rc::from(",")), Expr::String(Rc::from("a.b"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("replace")), Expr::Builtin(builtins::replace));

        interpriter.eval().unwrap();
    }
    
    #[test]
    #[should_panic]
    pub fn test_replace_bad_arg2() {
        let mut interpriter = Interpreter::new_vector_ast(vec![Expr::FnCall(ExprFnCall { name: Rc::from("replace"), args: vec![Expr::Symbol(Rc::from(".")), Expr::Boolean(true), Expr::String(Rc::from("a.b"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("replace")), Expr::Builtin(builtins::replace));

        interpriter.eval().unwrap();
    }
}
