use std::{io::stdin, rc::Rc};

use eval::Value;
use syntax::Expr;

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
fn test_parse_command() {
    let mut env = eval::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str("val 3");
    let command = grammar::command(&mut input).unwrap();
    let command = syntax::command_from_grammar(&command);
    println!("{command}");
    let output = env.eval_cmd(&command).unwrap();
    // let output = env.eval_expr(
    //     &Expr::String("3".into())
    // ).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
}

#[test]
fn test_gc_expr() {
    let mut env = eval::Env::new(gc::Strategy::Aggressive);
    let output = env.eval_expr(
        &Expr::String("3".into())
    ).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
}

#[test]
fn test_gc() {
    let mut env = eval::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str("
        val 3
    ");
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::command_from_grammar(&commands[0]);
    let output = env.eval_cmd(&commands).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
}

#[test]
fn test_closure_args() {
    let mut env = eval::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str("
        var double (add $1 $1)
        var 1 20
        double 3
    ");
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "6");
}

#[test]
fn test_closure() {
    let string = "
        # This is a comment.
        # The second line of a comment.
        var counter (
            var count 0
            val (
                set count $(inc $count)
                get count
            )
        )
        var c1 $(counter)
        c1
        var c2 $(counter)
        add $(c2) $(c1)

        # File is allowed to have trailing comments, apparently.
    ";
    let mut input = syntax::input_from_str(string);
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    println!("commands:");
    for command in &commands.0 {
        println!("{command}")
    }
    let mut env = eval::Env::new(gc::Strategy::Aggressive);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
}

#[test]
fn test_eval() {
    let mut env = eval::Env::new(gc::Strategy::Disabled);
    // let stack = env.gc.get_mut(env.stack);
    // stack.0.push(eval::Frame { variables: HashMap::new() });
    let mut input = syntax::input_from_str(
        "
            var var-name x
            var $var-name 1
            val $(
                set x 2
                set x $(inc $x)
                var tmp hello
                del tmp
                val $(
                    var x 10
                    set x 4
                )
                get x
            )
            )
        ",
    );
    let commands = grammar::multiline_commands(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let result = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    match env.gc.get(result) {
        Value::String(s) => assert_eq!(s, "3"),
        _ => unreachable!(),
    }
}

fn main() {
    let mut iter = (Box::new(chars()) as Box<dyn Iterator<Item = char>>).peekable();

    let mut env = eval::Env::new(gc::Strategy::Disabled);
    // {
    //     let mut stack = env.gc.get_mut(env.stack);
    //     stack.0.push(eval::Frame {
    //         variables: HashMap::new(),
    //     });
    // }
    loop {
        if let Some(command) = grammar::shell(&mut iter) {
            let command = syntax::command_from_grammar(&command);
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
                Ok(v) => match env.gc.get(v) {
                    Value::String(s) => println!("{s}"),
                    Value::Builtin(_f) => println!("<built-in fn>"),
                    Value::Closure { .. } => println!("<closure>"),
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
