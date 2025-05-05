use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, Read, stdin},
    rc::Rc,
};

use eval::{Env, Value};

mod eval;
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

#[test]
fn test_eval() {
    let mut env = eval::Env::new();
    // let stack = env.gc.get_mut(env.stack);
    // stack.0.push(eval::Frame { variables: HashMap::new() });
    let mut input = syntax::input_from_str(
        "
            var var-name x
            var $var-name 1
            val $(
                set x 3
                var tmp hello
                del tmp
                val $(
                    var x 10
                    set x 4
                )
                get x
            )
            )
        "
    );
    let commands = grammar::multiline_commands(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(commands);
    let result = env.eval_expr(&syntax::Expr::Block(commands)).unwrap();
    match env.gc.get(result) {
        Value::String(s) => assert_eq!(s, "3"),
        _ => unreachable!()
    }
}

fn main() {
    let mut iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable();

    let mut env = eval::Env::new();
    // {
    //     let mut stack = env.gc.get_mut(env.stack);
    //     stack.0.push(eval::Frame {
    //         variables: HashMap::new(),
    //     });
    // }
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
            match env.eval_cmd(&command) {
                Err(e) => {
                    println!("error:");
                    e.iter().for_each(|v| println!("{v}"));
                    continue;
                }
                Ok(v) => {
                    match env.gc.get(v) {
                        Value::String(s) => println!("{s}"),
                        Value::Builtin(_f) => println!("<built-in fn>"),
                        Value::Closure { .. } => println!("<closure>"),
                    }
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
