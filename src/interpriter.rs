use std::collections::BTreeMap;
use std::rc::Rc;
use crate::parser::{Expr, FnResultExpr};

mod builtins;

pub struct Interpreter {
    input: InterpreterInput,
    pos: usize,

    // this exists mostly to allow us to disable lazy eval for automated testing purposes. It changes very little else.
    // this should, broadly, never be actually set for non-testing code.
    pub(crate) disable_lazy: bool,

    pub(crate) env: Box<Env>,
}

#[derive(Debug, PartialEq, Hash, Clone)]
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
            Expr::Symbol(sym) => self,
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
            Expr::Symbol(symbol) => self.variables.insert(symbol, value),
            Expr::MapRef(name, attr) => {
                // Todo
                // We need to actually implement this
                match name {
                    Expr::Symbol(name) => if let Some(map) = self.get_variable(&name) {
                    } else {
                        panic!("Map {} does not exist!", name);
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
            Some((K, V)) => Some(V.clone()),
            None => match &self.parent {
                Some(p) => Some(p.find_variable(name)),
                None => None,
            }
        }
    }

    pub fn find_variable(&self, name: &Rc<str>) -> Expr {
        match self.get_variable(name) {
            Some(T) => T,
            None => panic!("Variable not found!");
        }
    }
}

pub enum InterpreterInput {
    VecAst(Vec<Expr>),
    ArrAst(Rc<[Expr]>),
}

impl Interpreter {
    pub fn new_vexprs(input: Vec<Expr>) -> Self{
       Self {
           input: InterpreterInput::VecAst(input),
           pos: 0,

           disable_lazy: false,

           env: Box::from(Env::new()),
       }
    }

    pub fn new(input: Rc<[Expr]>) -> Self {
        Self {
            input: InterpreterInput::ArrAst(input),
            pos: 0,

            disable_lazy: false,

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
        self.eval_(self.get_input())
    }

    pub fn eval_(&mut self, input: Expr) -> Option<Expr> {
        eval(input, &mut self.env, self.disable_lazy)
    }
}
pub fn eval(input: Expr, env: &mut Env, disable_lazy: bool) -> Option<Expr> {
    match input {
        Expr::VarDecl(name, mut value) => {
            env.add_variable(*name, *value.clone());

            Some(*value)
        },

        Expr::Symbol(sym) => {
            Some(env.find_variable(&sym))
        },

        Expr::ListRef(sym, index) => {
            let value = env.find_variable_with_expr(&sym);
            match value {
                Expr::List(list) => match list.get(index.num as usize){
                    Some(T) => Some(T.clone()),
                    None => panic!("Invalid index"),
                },
                _ => panic!("Unable to list access into a non-list!"),
            }
        },
        Expr::MapRef(sym, attr) => {
            let value = env.find_variable_with_expr(&sym);
            match value {
                Expr::Map(map) => {
                    for k in map {
                        if k.is_key(&sym) {
                            return Some(k.value)
                        }
                    }
                    panic!("Map Attr not found!")
                }
                _ => panic!("Unable to list access into a non-list!"),
            }
        },

        Expr::FnResult(expr) => {
            let FnResultExpr { function: f, args: args, env: env } = expr;
            f(args, &mut env.clone())
        },
        Expr::FnCall(fncall) => {
            let function = env.find_variable(&fncall.name);
            match function {
                Expr::Builtin(cb) => {
                    if disable_lazy {
                        eval(Expr::FnResult(FnResultExpr {
                            function: cb,
                            args: fncall.args,
                            env: env.clone(),
                        }), env, disable_lazy)
                    } else {
                        Some(Expr::FnResult(FnResultExpr {
                            function: cb,
                            args: fncall.args,
                            env: env.clone(),
                        }))
                    }
                },
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
    use crate::parser::{ExprFnCall, MapAttrExpr, NumberExpr};
    use super::*;

    #[test]
    pub fn test_evaluation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::Boolean(true)]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true))
    }

    #[test]
    pub fn test_evaluation_with_advance() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::Boolean(true)]);

        interpriter.advance();
        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true))
    }

    #[test]
    pub fn test_evaluation_with_advance_multiple() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::Boolean(true), Expr::String(Rc::from("This is a string"))]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
        assert_eq!(interpriter.advance().eval().unwrap(), Expr::String(Rc::from("This is a string")));
    }

    #[test]
    pub fn test_vardecl_evaulation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::VarDecl(Box::from(Expr::Symbol(Rc::from("test"))), Box::from(Expr::Boolean(true)))]);

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
        assert_eq!(interpriter.env.variables[&Rc::from("test")], Expr::Boolean(true));
    }

    #[test]
    pub fn test_symbol_evaulation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::Symbol(Rc::from("test"))]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::Number(NumberExpr::from_number(1.0)));

        assert_eq!(interpriter.eval().unwrap(), Expr::Number(NumberExpr::from_number(1.0)));
    }

    #[test]
    pub fn test_list_index_evaulation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::ListRef(Rc::from(Expr::Symbol(Rc::from("test"))), NumberExpr { num: 0.0 })]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::List(vec![Expr::Boolean(true)]));

        assert_eq!(interpriter.eval().unwrap(), Expr::Boolean(true));
    }

    #[test]
    pub fn test_map_access_evaluation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::MapRef(Rc::from(Expr::Symbol(Rc::from("test"))), Box::from(Expr::Symbol(Rc::from("test"))))]);
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::Map(vec![MapAttrExpr { key: Expr::Symbol(Rc::from("test")), value: Expr::Path(PathBuf::from("/home")) }]));

        assert_eq!(interpriter.eval().unwrap(), Expr::Path(PathBuf::from("/home")));
    }

    #[test]
    pub fn test_fncall_evaluation() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("add"), args: vec![Expr::Number(NumberExpr { num: 1.0 }), Expr::Number(NumberExpr { num: 1.0 })]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("add")), Expr::Builtin(builtins::add));

        assert_eq!(interpriter.eval().unwrap(), Expr::Number(NumberExpr { num: 1.0 + 1.0 }));
    }

    #[test]
    pub fn test_ghr_builtin_simple() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::String(Rc::from("test")), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2")} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::Symbol(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test2")), Expr::String(Rc::from("test2")));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2")} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols_nested() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::Symbol(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test2")), Expr::Symbol(Rc::from("test3")));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test3")), Expr::String(Rc::from("test2")));


        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2")} );
    }

    #[test]
    pub fn test_ghr_builtin_symbols_mixed() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));
        interpriter.env.add_variable(Expr::Symbol(Rc::from("test")), Expr::String(Rc::from("test")));

        assert_eq!(interpriter.eval().unwrap(), Expr::GitHubRemote { user: Rc::from("test"), repo: Rc::from("test2")} );
    }

    #[test]
    #[should_panic]
    pub fn test_ghr_builtin_bad_args() {
        let mut interpriter = Interpreter::new_vexprs(vec![Expr::FnCall(ExprFnCall { name: Rc::from("gh-r"), args: vec![Expr::Boolean(true), Expr::String(Rc::from("test2"))]})]);
        interpriter.disable_lazy = true;
        interpriter.env.add_variable(Expr::Symbol(Rc::from("gh-r")), Expr::Builtin(builtins::github_repo));

       interpriter.eval();
    }
}
