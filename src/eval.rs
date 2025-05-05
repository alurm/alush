use std::collections::HashMap;

use crate::{
    gc::{self, Gc},
    syntax::{self, Expr},
};

pub enum Value {
    String(String),
    Builtin(fn(&mut Env, args: Vec<Gc<Value>>) -> Result),
    Closure {
        code: syntax::Commands, 
        stack: Gc<Stack2>,
        // stack: Stack2,
    },
}

impl gc::Trace for Value {
    fn trace(&self) -> Vec<gc::Id> {
        match self {
            Value::String(_) => Vec::new(),
            Value::Builtin(_) => Vec::new(),
            Value::Closure { code: _, stack } => vec![stack.id],
        }
    }
}

pub struct Frame {
    pub variables: HashMap<String, Gc<Value>>,
}

pub struct Stack2 {
    frame: Frame,
    up: Option<Gc<Stack2>>,
}

impl gc::Trace for Stack2 {
    fn trace(&self) -> Vec<gc::Id> {
        let mut vec = self.frame.trace();
        if let Some(up) = self.up {
            vec.push(up.id);
        }
        vec
    }
}

// pub struct Stack(pub Vec<Frame>);

impl gc::Trace for Frame {
    fn trace(&self) -> Vec<gc::Id> {
        let mut vec = Vec::new();
        for (_, v) in &self.variables {
            vec.push(v.id);
        };
        vec
    }
}

// impl gc::Trace for Stack {
//     fn trace(&self) -> Vec<gc::Id> {
//         let mut result = Vec::new();
//         for value in &self.0 {
//             for (_, value) in &value.variables {
//                 result.push(value.id);
//             }
//         };
//         result
//     }
// }

// impl gc::Trace for RefCell<Stack> {
//     fn trace(&self) -> Vec<gc::Id> {
//         let mut result = Vec::new();
//         for value in &self.borrow().0 {
//             for (_, value) in &value.variables {
//                 result.push(value.id);
//             }
//         };
//         result
//     }
// }

pub struct Env {
    pub gc: gc::Heap,
    // pub stack: Stack,
    pub stack: Option<Gc<Stack2>>,
    // pub stack: Gc<Stack2>,
    // pub stack: Gc<Stack>,
}

fn builtin_set(env: &mut Env, args: Vec<Gc<Value>>) -> Result {
    let [name, value] = args[..] else {
        return Err(vec!["set <name> <value>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["set <name: string> <value>".into()])
    };
    env.update(&name.to_owned(), value);
    Ok(env.gc.alloc(Value::String("".into())))
}

fn builtin_val(env: &mut Env, tail_values: Vec<Gc<Value>>) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["val <value>".into()]);
    };
    Ok(value)
}

fn builtin_var(env: &mut Env, tail_values: Vec<Gc<Value>>) -> Result {
    let [name, value] = tail_values[..] else {
        return Err(vec!["var <name> <value>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["set <name: string> <value>".into()])
    };
    // else {
    //     return Err(vec!["var <name: string> <value>".into()]);
    // };

    let name = name.into();

    let stack = env.gc.get_mut(env.stack.unwrap());

    stack.frame.variables.insert(name, value);

    Ok(env.gc.alloc(Value::String("".into())))
}

fn builtin_get(env: &mut Env, tail_values: Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["get <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["get <name: string>".into()]); };
    let name = name.to_owned();
    let Some(value) = env.lookup(&name) else {
        return Err(vec!["get: var not found".into()]);
    };
    Ok(value)
}

fn builtin_del(env: &mut Env, tail_values: Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["del <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["del <name: string>".into()]); };
    let name = name.to_owned();
    if !env.forget(&name) {
        return Err(vec!["del: var not found".into()]);
    };
    Ok(env.gc.alloc(Value::String("".into())))
}

fn builtin_inc(env: &mut Env, tail_values: Vec<Gc<Value>>) -> Result {
    let [value] = tail_values[..] else { return Err(vec!["inc <number>".into()]) };
    let Value::String(value) = env.gc.get(value) else { return Err(vec!["inc <number: string>".into()]); };
    let Ok(mut n) = value.parse::<u8>() else {
        return Err(vec!["inc: parse failed".into()])
    };
    n = n + 1;
    let str = format!("{n}");
    Ok(env.gc.alloc(Value::String(str)))
    // let name = name.to_owned();
    // if !env.forget(&name) {
    //     return Err(vec!["del: var not found".into()]);
    // };
    // Ok(env.gc.alloc(Value::String("".into())))
}

impl Env {
    pub fn new() -> Self {
        let mut gc = gc::Heap::new(gc::Strategy::Disabled);
        let variables = HashMap::from([
            ("set".into(), gc.alloc(Value::Builtin(builtin_set))),
            ("get".into(), gc.alloc(Value::Builtin(builtin_get))),
            ("val".into(), gc.alloc(Value::Builtin(builtin_val))),
            ("var".into(), gc.alloc(Value::Builtin(builtin_var))),
            ("del".into(), gc.alloc(Value::Builtin(builtin_del))),
            ("inc".into(), gc.alloc(Value::Builtin(builtin_inc))),
        ]);
        let stack = Some(gc.alloc(Stack2 { frame: Frame { variables }, up: None }));
        // let stack = gc.alloc(Stack(Vec::new()));
        Env { gc, stack }
    }

    fn lookup(&mut self, name: &str) -> Option<Gc<Value>> {
        let mut maybe_stack = self.stack;
        while let Some(stack) = maybe_stack {
            let stack = self.gc.get(stack);
            if let Some(value) = stack.frame.variables.get(name) {
                return Some(*value);
            }
            maybe_stack = stack.up;
        }
        None
    }

    fn update<'a>(&mut self, name: &str, value: Gc<Value>) -> bool {
        let mut maybe_stack = self.stack;
        while let Some(stack) = maybe_stack {
            let stack = self.gc.get_mut(stack);
            if let Some(slot) = stack.frame.variables.get_mut(name) {
                *slot = value;
                return true;
            }
            maybe_stack = stack.up;
        }
        false
    }    

    fn forget<'a>(&mut self, name: &str) -> bool {
        let mut maybe_stack = self.stack;
        while let Some(stack) = maybe_stack {
            let stack = self.gc.get_mut(stack);
            if let Some(_) = stack.frame.variables.remove(name) {
                return true;
            }
            maybe_stack = stack.up;
        }
        false
    }   

    pub fn eval_cmd(&mut self, cmd: &syntax::Command) -> Result {
        let [head, tail @ ..] = &cmd.0[..] else {
            panic!();
        };

        // let args = tail.iter()
        //     .map(|expr| self.eval_expr(expr))
        //     .collect();
    
        let mut tail_values = Vec::new();
    
        for e in tail {
            tail_values.push(self.eval_expr(e)?);
        }

        // lookup if string
        // otherwise eval expr
        let head = match head {
            Expr::String(string) => {
                let Some(value)= self.lookup(string) else {
                    return Err(vec!["lookup failed".into()]);
                };
                value
            }
            _ => self.eval_expr(head)?,
        };

        match self.gc.get(head) {
            Value::String(_) => todo!(),
            Value::Builtin(f) => {
                f(self, tail_values)
            }
            Value::Closure { code, stack } => {
                // Wasteful clone.
                self.eval_closure(code.clone(), *stack)
            }
        }

        // match head {
        //     syntax::Expr::String(string) => {
        //         let Some(f) = self.lookup(string) else {
        //             return Err(vec!["lookup failed".into()]);
        //         };
        //         let f = self.gc.get(f);
        //         let Value::Builtin(f) = f else {
        //             return Err(vec!["cmd head is not fn".into()]);
        //         };
        //         match f {
        //             Value::Builtin(f) =>
        //                 return f(self, tail_values),
        //             Value::String(_) => return Err(vec!["cmd head is string"]),
        //             Value::Closure { code, stack } =>
        //                 return eval
        //         }
        //         // match string.as_ref() {
        //         //     "set" => return builtin_set(self, tail_values),
        //         //     "val" => return builtin_val(self, tail_values),
        //         //     "var" => return builtin_var(self, tail_values),
        //         //     "get" => return builtin_get(self, tail_values),
        //         //     _ => Err(vec!["unknown cmd".to_owned()])
        //         // }
        //     }
        //     any => {
        //         let value = self.eval_expr(any)?;
        //         let value = self.gc.get(value);
        //         match value {
        //             Value::String(_) => todo!(),
        //             Value::Closure { code, stack } => {
        //                 // self.eval_expr(expr)
        //                 // Clone is wasteful.
        //                 return self.eval_closure(code.clone(), *stack);
        //                 // todo!()
        //             }
        //             Value::Builtin(f) => {
        //                 return f(self, tail_values);
        //             }
        //         }
        //     }
        //     // syntax::Expr::Block(commands) => {
        //     //     let [.., last] = &commands.0[..] else {
        //     //         return Err(vec!["empty block".to_owned()]);
        //     //     };
        //     //     eval_cmd(env, last)
        //     // }
        //     _ => todo!()
        // }
    }

    pub fn eval_expr(&mut self, expr: &syntax::Expr) -> Result {
        match expr {
            syntax::Expr::String(s) => Result::Ok(self.gc.alloc(Value::String(s.to_owned()))),
            syntax::Expr::Block(commands) => {
                self.stack = Some(self.gc.alloc(Stack2 { frame: Frame { variables: HashMap::new() }, up: self.stack }));
                // // env.stack = 
                // env.gc.get_mut(env.stack.unwrap()).0.push(Frame { variables: HashMap::new() });
                let mut result = None;
                for command in &commands.0 {
                    result = Some(self.eval_cmd(&command));
                }
                // env.gc.get_mut(env.stack).0.pop().unwrap();
                self.stack = self.gc.get(self.stack.unwrap()).up;
                result.unwrap()
            }
            Expr::Closure(commands) => {
                Ok(self.gc.alloc(Value::Closure {
                    // Waste.
                    code: commands.clone(),
                    stack: self.stack.unwrap(),
                }))
            }
        }
    }

    // No idea if this is correct.
    fn eval_closure(&mut self, commands: syntax::Commands, stack: Gc<Stack2>) -> Result {
        let old_stack = self.stack;

        self.stack = Some(stack);

        let value = self.eval_expr(&syntax::Expr::Block(commands));

        self.stack = old_stack;

        value
    }
}

pub type Result = std::result::Result<Gc<Value>, Vec<String>>;

// #[test]
// #[ignore]
// fn test_eval() {
//     let mut gc = gc::Heap::new();
//     // let stack = gc.alloc(Stack(Vec::new()));
//     // gc.root(stack);
//     // crate::grammar::command(i)
// }
