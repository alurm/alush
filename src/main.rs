use std::io::{stdin, Write};

use shell::{grammar, interpreter::{self, print_error, Env}, syntax};

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
        print!("$ ");
        std::io::stdout().flush().unwrap();
        if iter.peek().is_none() { return }
        if let Some(command) = grammar::shell(&mut iter) {
            let command = syntax::command_from_grammar(&command);
            match env.eval_cmd(&command) {
                Err(e) => interpreter::print_error(e),
                Ok(v) => {
                    env.print_value(v);
                    env.gc.unroot(v);
                }
            }
        } else {
            println!("error: syntax error");
            // . ctrl-d to reset the buffer, ctrl-c to exit");
            drop(iter);
            iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable()
        }
    }
}

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

fn dofile(file: String) {
    let mut env = interpreter::Env::new(gc::Strategy::Default);
    let mut input = syntax::input_from_str(&file);
    let Some(file) = grammar::file(&mut input) else {
        println!("syntax error");
        return;
    };
    let commands = syntax::commands_from_grammar(&file);
    let mut result = None;
    for command in commands.0 {
        if let Some(result) = result {
            env.gc.unroot(result);
        }
        match env.eval_cmd(&command) {
            Err(e) => {
                print_error(e);
                return
            }
            Ok(v) => result = Some(v),
        }
    }
}

fn main() {
    // Produces an empty line on `./shell | true` for some reason?
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };

    let args = std::env::args();
    let args: Vec<_> = args.collect();
    if let Some(path) = args.get(1) {
        let Ok(file) = std::fs::read_to_string(path) else {
            println!("Failed to read path");
            return;
        };
        dofile(file);
    } else {
        shell();
    }
}
