use std::{cell::RefCell, io::{stdin, BufRead, Read}, rc::Rc};

use eval::{EnvNode, Eval};

mod grammar;
mod syntax;
mod print;
mod eval;
mod gc;

fn chars() -> impl Iterator<Item = char> {
    stdin().lines().flat_map(|l| l).flat_map(
        |s| {
            let mut chars = Vec::new();
            for c in s.chars() {
                chars.push(c);
            }
            chars.push('\n');
            chars
        }
    )
}

fn main() {
    let mut iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable();

    loop {
        if let Some(command) = grammar::shell(&mut iter) {
            let command = syntax::command_from_grammar(command);
            println!("got: {command}");
            let env: eval::Env = Some(Rc::new(RefCell::new(
                    EnvNode {
                        up: None,
                        variables: std::collections::HashMap::from([
                            ("greeting".to_string(), eval::Value::String("hi".to_string()))
                        ]),
                    }
            )));
            command.eval(env);
        } else {
            println!("syntax error. ctrl-d to reset the buffer, ctrl-c to exit");

            // drop(iter);
            // // Consume all available input and try again?
            // // ^D with no input chars will restart it.
            for _ in iter {}

            // Reset the buffer.
            // {
            //     let mut stdin = stdin().lock();
            //     if let Ok(buffer) = stdin.fill_buf() {
            //         let len = buffer.len();
            //         if len == 0 { return }
            //         stdin.consume(len);
            //     } else {
            //         return
            //     }
            // }

            iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable()
        }
    }
}
