use crate::syntax::{Command, Expr};

impl Command {
    fn pretty(&self, to: &mut String, depth: usize) {
        let mut first = true;
        for e in &self.0 {
            if !first {
                to.push(' ');
            }
            first = false;
            e.pretty(to, depth);
        }
    }
}

fn tab(to: &mut String, depth: usize) {
    for _ in 0..depth {
        to.push_str("    ");
    }
}

impl Expr {
    pub fn pretty(&self, to: &mut String, mut depth: usize) {
        match self {
            Expr::String(s) => {
                to.push('\'');
                for c in s.chars() {
                    if c == '\'' {
                        to.push('\'');
                        to.push('\'');
                    } else {
                        to.push(c);
                    }
                }
                to.push('\'');
            }
            Expr::Closure(commands) | Expr::Block(commands) => {
                if let Expr::Block(_) = self {
                    to.push('$');
                }
                to.push('(');
                depth += 1;
                match &commands.0[..] {
                    [] => (),
                    [c] => c.pretty(to, depth),
                    _ => {
                        for command in &commands.0 {
                            to.push('\n');
                            tab(to, depth);
                            command.pretty(to, depth);
                        }
                        to.push('\n');
                        tab(to, depth - 1);
                    }
                }
                // depth -= 1;
                to.push(')');
            }
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.pretty(&mut s, 0);
        f.write_str(&s)
    }
}

#[test]
fn test_command_from_grammar() {
    let original = "a b (
        c d
        e f
        (
            hello world
            more stuff
            simple
            ()
            complex (kek) $(
                block content
                more stuff
            )
        )
    )";

    let one = {
        let mut input = crate::syntax::input_from_str(original);
        let grammar = crate::grammar::command(&mut input).unwrap();
        let syntax = crate::syntax::command_from_grammar(&grammar);
        let mut string = String::new();
        syntax.pretty(&mut string, 0);
        string
    };

    let two = {
        let mut input = crate::syntax::input_from_str(&one);
        let grammar = crate::grammar::command(&mut input).unwrap();
        let syntax = crate::syntax::command_from_grammar(&grammar);
        let mut string = String::new();
        syntax.pretty(&mut string, 0);
        string
    };

    println!("{one}");

    assert_eq!(one, two);
}
