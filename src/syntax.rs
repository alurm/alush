pub type Input = std::iter::Peekable<Box<dyn Iterator<Item = char>>>;

pub struct Command(pub Vec<Expr>);

pub struct Commands(pub Vec<Command>);

pub enum Expr {
    String(String),
    // Rc is needed since closures need to own commands without cloning exprs expensively.
    Closure(Rc<Commands>),
    Block(Rc<Commands>),
}

use std::rc::Rc;

use crate::grammar;

pub fn command_from_grammar(g: &grammar::Command) -> Command {
    let mut c = Command(Vec::new());

    for e in g {
        c.0.push(expr_from_grammar(e))
    }

    c
}

pub fn commands_from_grammar(g: &grammar::Commands) -> Commands {
    let mut cs = Commands(Vec::new());

    for c in g {
        cs.0.push(command_from_grammar(c));
    }

    cs
}

pub fn expr_from_grammar(g: &grammar::Expr) -> Expr {
    match g {
        // (...) is a closure.
        grammar::Expr::Commands {
            dollar: false,
            value,
        } => Expr::Closure(Rc::new(commands_from_grammar(value))),
        // $(...) is a block.
        grammar::Expr::Commands {
            dollar: true,
            value,
        } => Expr::Block(Rc::new(commands_from_grammar(value))),
        // foo is a string.
        grammar::Expr::String {
            dollar: false,
            value,
        } => Expr::String(value.into()),
        // $x desugars to $(get x)
        grammar::Expr::String {
            dollar: true,
            value,
        } => {
            let mut command = Command(Vec::new());
            command.0.push(Expr::String(String::from("get")));
            command.0.push(Expr::String(value.into()));
            let commands = vec![command];
            Expr::Block(Rc::new(Commands(commands)))
        }
    }
}

pub fn input_from_str(s: &str) -> Input {
    let mut chars = Vec::<char>::new();

    for c in s.chars() {
        chars.push(c);
    }

    let heap = Box::new(chars.into_iter());

    let fat = heap as Box<dyn Iterator<Item = _>>;

    fat.peekable()
}
