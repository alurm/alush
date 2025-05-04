use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, Read, stdin},
    rc::Rc,
};

use eval::{EnvNode, Eval};

mod eval;
mod eval_2;
mod eval_3;
mod gc;
mod grammar;
mod print;
mod syntax;

fn chars() -> impl Iterator<Item = char> {
    stdin().lines().flat_map(|l| l).flat_map(|s| {
        let mut chars = Vec::new();
        for c in s.chars() {
            chars.push(c);
        }
        chars.push('\n');
        chars
    })
}

fn main() {
    let mut iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable();

    use crate::eval_3 as eval;
    let mut env = eval::Env::new();
    {
        let mut stack = env.gc.get(env.stack).borrow_mut();
        stack.0.push(eval::Frame {
            variables: HashMap::new(),
        });
    }
    loop {
        if let Some(command) = grammar::shell(&mut iter) {
            let command = syntax::command_from_grammar(command);
            println!("parse: {command}");
            // let env: eval::Env = Some(Rc::new(RefCell::new(
            //         EnvNode {
            //             up: None,
            //             variables: std::collections::HashMap::from([
            //                 ("greeting".to_string(), eval::Value::String("hi".to_string()))
            //             ]),
            //         }
            // )));
            // command.eval(env);
            match eval::eval_cmd(&mut env, &command) {
                Err(e) => {
                    println!("error:");
                    e.iter().for_each(|v| println!("{v}"));
                    continue;
                }
                Ok(v) => {
                    let eval::Value::String(string) = env.gc.get(v);
                    println!("eval: {string}");
                }
            };
            println!();
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
