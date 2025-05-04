use std::{cell::RefCell, collections::HashMap};

use crate::{gc::attempt_6::{self as gc, Gc}, syntax::{self, Expr}};

#[derive(Debug)]
pub enum Value {
    String(String),
}

impl gc::Trace for Value {
    fn trace(&self) -> Vec<gc::Id> {
        match self {
            Value::String(_) => Vec::new(),
        }
    }
}

pub struct Frame {
    pub variables: HashMap<String, Gc<Value>>,
}

pub struct Stack(pub Vec<Frame>);

impl gc::Trace for RefCell<Stack> {
    fn trace(&self) -> Vec<gc::Id> {
        let mut result = Vec::new();
        for value in &self.borrow().0 {
            for (_, value) in &value.variables {
                result.push(value.id);
            }
        };
        result
    }
}

pub struct Env {
    pub gc: gc::Heap,
    pub stack: Gc<RefCell<Stack>>,
}

impl Env {
    pub fn new() -> Self {
        let mut gc = gc::Heap::new(gc::Strategy::Disabled);
        let stack = gc.alloc(RefCell::new(Stack(Vec::new())));
        Env { gc, stack }
    }
}

fn lookup(env: &mut Env, name: &str) -> Option<Gc<Value>> {
    let stack = env.gc.get(env.stack).borrow();
    for frame in stack.0.iter().rev() {
        if let Some(value) = frame.variables.get(name) {
            return Some(*value);
        }
    }
    return None;
}

pub type Result = std::result::Result<Gc<Value>, Vec<String>>;

pub fn eval_expr(env: &mut Env, expr: &syntax::Expr) -> Result {
    match expr {
        syntax::Expr::String(s) => Result::Ok(env.gc.alloc(Value::String(s.to_owned()))),
        syntax::Expr::Block(commands) => eval_cmd(env, commands.0.last().unwrap()),
        _ => todo!(),
    }
}

pub fn eval_cmd(env: &mut Env, cmd: &syntax::Command) -> Result {
    let [head, tail @ ..] = &cmd.0[..] else {
        panic!();
    };

    let mut tail_values = Vec::new();

    for e in tail {
        tail_values.push(eval_expr(env, e)?);
    }

    match head {
        syntax::Expr::String(string) => {
            match string.as_ref() {
                "var" => {
                    {
                        let [name, value] = tail_values[..] else {
                            return Err(vec!["var <name> <value>".into()]);
                        };
                        let Value::String(name) = env.gc.get(name) else {
                            return Err(vec!["var <name: string> <value>".into()]);
                        };
                        let name = name.into();
                        let stack = env.gc.get(env.stack);
                        let mut stack = stack.borrow_mut();
                        let Some(frame) = stack.0.last_mut() else {
                            return Err(vec!["var: expected to have a stack frame".into()])
                        };
                        frame.variables.insert(name, value);
                    }
                    Ok(env.gc.alloc(Value::String("".into())))
                }
                "get" => {
                    let [name] = tail_values[..] else { return Err(vec!["get <name>".into()]) };
                    let Value::String(name) = env.gc.get(name) else { return Err(vec!["get <name: string>".into()]); };
                    let name = name.to_owned();
                    let Some(value) = lookup(env, &name) else {
                        return Err(vec!["get: var not found".into()]);
                    };
                    Ok(value)
                }
                _ => Err(vec!["unknown cmd".to_owned()])
            }
        }
        // syntax::Expr::Block(commands) => {
        //     let [.., last] = &commands.0[..] else {
        //         return Err(vec!["empty block".to_owned()]);
        //     };
        //     eval_cmd(env, last)
        // }
        _ => todo!()
    }
}

// #[test]
// #[ignore]
// fn test_eval() {
//     let mut gc = gc::Heap::new();
//     // let stack = gc.alloc(Stack(Vec::new()));
//     // gc.root(stack);
//     // crate::grammar::command(i)
// }
