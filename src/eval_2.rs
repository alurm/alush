use core::panic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{gc::{self, Gc}, syntax};

enum InnerValue {
    String(String),
    Builtin(fn(_: &Env) -> InnerValue),

    // This is workaround exists since otherwise Env would not be traced.
    Env(Env),
}

#[derive(Copy, Clone)]
struct Value(gc::Ptr<InnerValue>);

struct EnvNode {
    variables: HashMap<String, Value>,
    up: Option<Rc<RefCell<EnvNode>>>,
}

type Env = Option<Rc<RefCell<EnvNode>>>;

fn eval_cmd(gc: &Gc<InnerValue>, env: &Env, cmd: syntax::Command) {
    let head = &cmd.0[0];
    let tail = &cmd.0[1..];

    let head = match head {
        syntax::Expr::String(s) => lookup(&env, s),
        _ => eval_expr(&env, head),
    };

    match gc.get(head.0) {
        InnerValue::Builtin(builtin) => {
            // apply_builtin(builtin, tail);
            todo!()
        },
        _ => panic!()
    }

    // let head = match head.0. {

    // }

    let tail = tail.iter().map(|it| eval_expr(&env, it)).collect::<Vec<_>>();

    todo!()

    // lookup(env, head);
}

fn lookup(env: &Env, string: &str) -> Value {
    match env {
        Some(env) => {
            let env = env.borrow();
            match env.variables.get(string) {
                None => lookup(&env.up, string),
                Some(v) => *v,
            }
        },
        None => panic!()
    }
}

fn eval_expr(env: &Env, expr: &syntax::Expr) -> Value {
    match expr {
        syntax::Expr::String(s) => InnerValue::String(s.clone()),
        _ => todo!()
    };
    todo!()
}

// enum Env {

// }

// fn eval(cmd: syntax::Command) {
//     match cmd.0[0] {

//     }
// }
