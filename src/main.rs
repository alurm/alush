use std::io::stdin;

use eval::{Env, Value};

mod eval;
// mod gc;
mod grammar;
mod print;
mod syntax;

#[cfg(test)]
mod tests;

fn chars() -> impl Iterator<Item = char> {
    stdin().lines().map_while(Result::ok).flat_map(|s| {
        let mut chars = Vec::new();
        for c in s.chars() {
            chars.push(c);
        }
        chars.push('\n');
        chars
    })
}

fn shell() {
    let mut iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable();

    let mut env = Env::new(gc::Strategy::Disabled);
    // {
    //     let mut stack = env.gc.get_mut(env.stack);
    //     stack.0.push(eval::Frame {
    //         variables: HashMap::new(),
    //     });
    // }
    loop {
        if let Some(command) = grammar::shell(&mut iter) {
            let command = syntax::command_from_grammar(&command);
            // println!("parse: {command}");
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
                    print!("error: ");
                    e.iter().for_each(|v| println!("{v}"));
                    println!();
                    continue;
                }
                Ok(v) => match env.gc.get(v) {
                    Value::String(s) => println!("{s}"),
                    Value::Builtin(_f) => println!("<built-in fn>"),
                    Value::Closure { .. } => println!("<closure>"),
                    Value::Throw(_) => {
                        println!("<throw ...>");
                    },
                    Value::Lazy(_) => println!("<lazy>"),
                    Value::Map(_) => println!("<map>"),
                },
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

fn dofile(file: String) {
    let mut env = eval::Env::new(gc::Strategy::Default);
    let mut input = syntax::input_from_str(&file);
    let Some(file) = grammar::file(&mut input) else {
        println!("Syntax error");
        return
    };
    let commands = syntax::commands_from_grammar(&file);
    for command in commands.0 {
        match env.eval_cmd(&command) {
            Err(error) => {
                println!("Error!");
                for e in error {
                    println!("{e}")
                }
                return
            }
            Ok(value) => {
                env.gc.unroot(value);
            }
        }
    }
}

fn main() {
    let args = std::env::args();
    let args: Vec<_> = args.collect();
    if let Some(path) = args.get(1) {
        let Ok(file) = std::fs::read_to_string(path) else {
            println!("Failed to read path");
            return
        };
        dofile(file);
    } else {
        shell();
    }
}