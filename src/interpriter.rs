use crate::parser::Expr;

pub struct Interpreter {
    input: InterpreterInput,
    pos: usize,
}

pub enum InterpreterInput {
    Ast(Vec<Expr>)
}

impl Interpreter {
    pub fn new_vexprs(input: Vec<Expr>) -> Self{
       Self {
           input: InterpreterInput::Ast(input),
           pos: 0
       }
    }

    pub fn advance(&mut self) -> &Self {
        self.pos += 1;
        self
    }

    pub fn get_input(&self) -> Expr {
        match &self.input {
            InterpreterInput::Ast(vec) => vec[self.pos].clone()
        }
    }

    pub fn evaulate(&mut self) -> &Self {
        match self.get_input() {
            Expr::String(_) => {}
            Expr::Number(_) => {}
            Expr::Boolean(_) => {}
            Expr::Symbol(_) => {}
            Expr::Path(_) => {}
            Expr::VarDecl(_, _) => {}
            Expr::List(_) => {}
            Expr::ListRef(_, _) => {}
            Expr::Map(_) => {}
            Expr::MapRef(_, _) => {}
            Expr::FnCall(_) => {}
        }
        self
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn two_plus_two() {
        assert_eq!(2+2, 4)
    }

    #[test]
    #[should_panic]
    pub fn panic_two_plus_two() {
        assert_eq!(2+2, 5)
    }
}
