use std::rc::Rc;

use crate::{interpreter::{Env, Value}, syntax::Expr};
use super::*;

#[test]
fn test_apply() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let mut input = syntax::input_from_str(
        "
        var six $(apply $+ 1 2 3)
        var seven $(apply (+ $1 $2) 3 4)
        + $six $seven
    ",
    );
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let output = env.eval_expr(&Expr::Block(Rc::new(commands))).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "13");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_unix() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let mut input = syntax::input_from_str(
        "
        unix echo -n hi
    ",
    );
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let output = env.eval_expr(&Expr::Block(Rc::new(commands))).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "hi");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_fail() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let mut input = syntax::input_from_str(
        "
        fail
    ",
    );
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let Err(_) = env.eval_expr(&Expr::Block(Rc::new(commands))) else { unreachable!() };
}

#[test]
fn test_maps() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let mut input = syntax::input_from_str(
        "
        var key-2 ok
        var key-left ok
        var m $(map $key-2 ok $key-left ok)
        var key hello
        m set $key world
        $(if $(= $(m has $key) false) (throw) ())
        $(if $(= $(m has $key-2) false) (throw) ())
        m del doesn''t-exist
        m del $key-2
        m get $key
    ",
    );
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let output = env.eval_expr(&Expr::Block(Rc::new(commands))).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "world");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_gc() {
    let mut env = interpreter::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str(
        "
        val 3
    ",
    );
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
    let mut env = interpreter::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str(
        "
        var double (+ $1 $1)
        var 1 20
        double 3
    ",
    );
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
fn test_precise_gc() {
    let _commands = {
        let string = "
            val 3
        ";
        let mut input = syntax::input_from_str(string);
        let commands = grammar::file(&mut input).unwrap();
        syntax::commands_from_grammar(&commands)
    };
    let command = {
        let string = "val 3";
        let mut input = syntax::input_from_str(string);
        let command = grammar::command(&mut input).unwrap();
        syntax::command_from_grammar(&command)
    };

    let mut env = Env::new(gc::Strategy::Checking);
    // env.eval_expr(&syntax::Expr::Block(Rc::new(commands)));
    let id = env.eval_cmd(&command).unwrap();
    env.gc.unroot(id);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_precise_catch_loop_throw() {
    let commands = {
        let string = "
            var count 0
            val $(catch $(
                repeat $(
                    println $count
                    set count $(+ 1 $count)
                    $(if $(= $count 10) (throw $count) ())
                )
            ))
        ";
        let mut input = syntax::input_from_str(string);
        let commands = grammar::file(&mut input).unwrap();
        syntax::commands_from_grammar(&commands)
    };

    let mut env = Env::new(gc::Strategy::Checking);
    let id = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    // let id = env.eval_cmd(&command).unwrap();
    let Value::String(string) = env.gc.get(id) else {
        panic!()
    };
    assert_eq!(string, "10");
    env.gc.unroot(id);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(env.gc.roots.len(), 0);
    assert_eq!(env.gc.map.len(), 0);
}


#[test]
fn test_factorial_example() {
    let string = include_str!("../examples/factorial.lang");
    let mut input = syntax::input_from_str(string);
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    println!("commands:");
    for command in &commands.0 {
        println!("{command}")
    }
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let stack = env.gc.get(env.stack);
    let result = stack.frame.variables.get("var").unwrap();
    let Value::String(s) = env.gc.get(*result) else {
        panic!()
    };
    assert_eq!(s, "120");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(env.gc.roots.len(), 0);
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_factorial() {
    let string = "
        var factorial (
            var x $1

            if $(= $x 0) \\
                $(val 1)
                $(* $x $(factorial $(+ $x -1)))
            ;
        )

        factorial 5
    ";
    let mut input = syntax::input_from_str(string);
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    println!("commands:");
    for command in &commands.0 {
        println!("{command}")
    }
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "120");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(env.gc.roots.len(), 0);
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_multiline_command() {
    let string = "
        val \\
            3
        ;
    ";
    let mut input = syntax::input_from_str(string);
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    println!("commands:");
    for command in &commands.0 {
        println!("{command}")
    }
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(env.gc.roots.len(), 0);
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_closure() {
    let string = "
        # This is a comment.
        # The second line of a comment.
        var counter (
            var count 0
            var count 0
            val (
                set count $(inc $count)
                get count
            )
        )
        var c1 $(counter)
        c1
        var c2 $(counter)
        + $(c2) $(c1)

        # File is allowed to have trailing comments, apparently.
    ";
    let mut input = syntax::input_from_str(string);
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    println!("commands:");
    for command in &commands.0 {
        println!("{command}")
    }
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let output = env
        .eval_expr(&syntax::Expr::Block(Rc::new(commands)))
        .unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}

#[test]
fn test_eval() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
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
    let Value::String(s) = env.gc.get(result) else {
        unreachable!()
    };
    assert_eq!(s, "3");
}

#[test]
fn test_parse_command() {
    let mut env = interpreter::Env::new(gc::Strategy::Aggressive);
    let mut input = syntax::input_from_str("val 3");
    let command = grammar::command(&mut input).unwrap();
    let command = syntax::command_from_grammar(&command);
    println!("{command}");
    let output = env.eval_cmd(&command).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert_eq!(s, "3");
}

#[ignore]
#[test]
fn test_map_each() {
    let mut env = interpreter::Env::new(gc::Strategy::Checking);
    let mut input = syntax::input_from_str(
        "
        var m $(map name John age 40)
        var output ''
        m each (set output $(.. $output $1 ': ' $2 '; '))
        val $output
    ",
    );
    let commands = grammar::file(&mut input).unwrap();
    let commands = syntax::commands_from_grammar(&commands);
    let output = env.eval_expr(&Expr::Block(Rc::new(commands))).unwrap();
    let Value::String(s) = env.gc.get(output) else {
        panic!()
    };
    assert!(s == "name: John; age: 40; " || s == "age: 40; name: John; ");
    env.gc.unroot(output);
    env.gc.unroot(env.stack);
    env.gc.collect();
    assert_eq!(0, env.gc.roots.len());
    assert_eq!(0, env.gc.map.len());
}
