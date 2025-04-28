pub type Input = std::iter::Peekable<Box<dyn Iterator<Item = char>>>;

#[derive(Debug)]
pub struct Command(pub Vec<Expr>);

#[derive(Debug)]
pub struct Commands(pub Vec<Command>);

#[derive(Debug)]
pub enum Expr {
    String(String),
    Closure(Commands),
    Block(Commands),
}

use crate::grammar;

pub fn command_from_grammar(g: grammar::Command) -> Command {
    let mut c = Command(Vec::new());

    for e in g {
        c.0.push(expr_from_grammar(e))
    }

    c
}

pub fn commands_from_grammar(g: grammar::Commands) -> Commands {
    let mut cs = Commands(Vec::new());

    for c in g {
        cs.0.push(command_from_grammar(c));
    }

    cs
}

pub fn expr_from_grammar(g: grammar::Expr) -> Expr {
    match g {
        // (...) is a closure.
        grammar::Expr::Commands {
            dollar: false,
            value,
        } => Expr::Closure(commands_from_grammar(value)),
        // $(...) is a block.
        grammar::Expr::Commands {
            dollar: true,
            value,
        } => Expr::Block(commands_from_grammar(value)),
        // foo is a string.
        grammar::Expr::String {
            dollar: false,
            value,
        } => Expr::String(value),
        // $x desugars to $(get x)
        grammar::Expr::String {
            dollar: true,
            value,
        } => {
            let mut command = Command(Vec::new());
            command.0.push(Expr::String(String::from("get")));
            command.0.push(Expr::String(value));
            let commands = vec![command];
            Expr::Block(Commands(commands))
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

#[test]
#[ignore]
fn a_test() {
    let mut input = input_from_str("
        # Hello, world
        # Very cool
        a b c
        d $(e f g)
        (
            # Recursion
        )
        )
    ");
    dbg!(grammar::multiline_commands(&mut input));
}
