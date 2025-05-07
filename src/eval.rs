use std::{collections::HashMap, rc::Rc};

use crate::{
    gc::{self, Gc},
    syntax::{self, Expr},
};

pub enum Value {
    String(String),
    Builtin(fn(&mut Env, args: &Vec<Gc<Value>>) -> Result),
    Closure {
        code: Rc<syntax::Commands>, 
        stack: Gc<Stack>,
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

pub struct Stack {
    frame: Frame,
    up: Option<Gc<Stack>>,
}

impl gc::Trace for Stack {
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
    pub stack: Gc<Stack>,
}

fn builtin_set(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [name, value] = args[..] else {
        return Err(vec!["set <name> <value>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["set <name: string> <value>".into()])
    };
    env.update(&name.to_owned(), value);
    Ok(env.gc.rooted(Value::String("".into())))
}

fn builtin_val(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["val <value>".into()]);
    };
    Ok(value)
}

fn builtin_var(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
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

    let stack = env.gc.get_mut(env.stack);

    stack.frame.variables.insert(name, value);

    Ok(env.gc.rooted(Value::String("".into())))
}

fn builtin_get(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["get <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["get <name: string>".into()]); };
    let name = name.to_owned();
    let Some(value) = env.lookup(&name) else {
        return Err(vec!["get: var not found".into()]);
    };
    Ok(value)
}

fn builtin_del(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["del <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["del <name: string>".into()]); };
    let name = name.to_owned();
    if !env.forget(&name) {
        return Err(vec!["del: var not found".into()]);
    };
    Ok(env.gc.rooted(Value::String("".into())))
}

fn builtin_inc(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [value] = tail_values[..] else { return Err(vec!["inc <number>".into()]) };
    let Value::String(value) = env.gc.get(value) else { return Err(vec!["inc <number: string>".into()]); };
    let Ok(n) = value.parse::<i32>() else {
        return Err(vec!["inc: parse failed".into()])
    };
    let str = format!("{}", n + 1);
    Ok(env.gc.rooted(Value::String(str)))
    // let name = name.to_owned();
    // if !env.forget(&name) {
    //     return Err(vec!["del: var not found".into()]);
    // };
    // Ok(env.gc.alloc(Value::String("".into())))
}

fn builtin_add(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let mut sum = 0;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else { return Err(vec!["add <number: string>...".into()])};
        let Ok(number) = value.parse::<i32>() else {
            return Err(vec!["add: parse failed".into()]);
        };
        sum += number;
    };
    let string = format!("{sum}");
    Ok(env.gc.rooted(Value::String(string)))
}

impl Env {
    pub fn new(strategy: gc::Strategy) -> Self {
        let mut gc = gc::Heap::new(strategy);

        let builtins = [
            (
                "set",
                builtin_set as
                fn(&mut Env, &Vec<Gc<Value>>) -> Result
            ),
            ("get", builtin_get),
            ("val", builtin_val),
            ("var", builtin_var),
            ("del", builtin_del),
            ("inc", builtin_inc),
            ("add", builtin_add),
        ];

        let builtins = builtins
            .map(|(k, v)| {
                (k.into(), gc.rooted(Value::Builtin(v)))
            });

        // Annoying clone here.
        let variables = HashMap::from(builtins.clone());

        let stack = gc.rooted(Stack {
            frame: Frame { variables },
            up: None
        });

        builtins.iter().for_each(|(_, v)| {
            gc.unroot(*v);
        });
        
        // let stack = gc.rooted(Stack { frame: Frame { variables }, up: None });
        Env { gc, stack }
    }

    fn lookup(&mut self, name: &str) -> Option<Gc<Value>> {
        let mut maybe_stack = Some(self.stack);
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
        let mut maybe_stack = Some(self.stack);
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
        let mut maybe_stack = Some(self.stack);
        while let Some(stack) = maybe_stack {
            let stack = self.gc.get_mut(stack);
            if let Some(_) = stack.frame.variables.remove(name) {
                return true;
            }
            maybe_stack = stack.up;
        }
        false
    }   

    // Roots result
    pub fn eval_cmd(&mut self, cmd: &syntax::Command) -> Result {
        let [head, tail @ ..] = &cmd.0[..] else {
            panic!();
        };

        let mut tail_values = Vec::new();
    
        for e in tail {
            // all rooted, need to unroot
            tail_values.push(self.eval_expr(e)?);
        }

        // lookup if string
        // otherwise eval expr
        let head = match head {
            Expr::String(string) => {
                let Some(value)= self.lookup(string) else {
                    return Err(vec!["lookup failed".into()]);
                };
                // need to root so doesn't disappear during eval? idk.
                self.gc.root(value);
                value
            }
            _ => self.eval_expr(head)?, // rooted, need to unroot later
        };

        match self.gc.get(head) {
            // errors should ideally cause unroots to happen?
            Value::String(_) => Err(vec!["cmd's fn must not be a string".into()]),
            Value::Builtin(f) => {
                let result = f(self, &tail_values);
                // unroot here on in builtins?
                for e in tail_values {
                    self.gc.unroot(e);
                }
                result
            }
            // stack should be reachable via closure.
            Value::Closure { code, stack } => {
                let result = self.eval_closure(
                    code.clone(), *stack, &tail_values
                );
                for e in tail_values {
                    self.gc.unroot(e);
                }
                result
            }
        }
    }

    // No idea if this is correct.
    fn eval_closure(&mut self, commands: Rc<syntax::Commands>, stack: Gc<Stack>, args: &Vec<Gc<Value>>) -> Result {    
        let old_stack = self.stack;

        // I think stack should be reachable via closure so it should be fine to use it.
    
        // Append args as new stack frame.

        let new_stack = self.gc.rooted(Stack {
            frame: Frame { variables: HashMap::new() },
            up: Some(stack),
        });

        let new_stack_value = self.gc.get_mut(new_stack);

        for (i, &arg) in args.iter().enumerate() {
            // Start from $1. Mostly arbitrary.
            let str = format!("{}", i + 1);
            new_stack_value.frame.variables.insert(str, arg);
        }

        self.stack = new_stack;

        let value = self.eval_expr(&syntax::Expr::Block(commands));

        self.gc.unroot(new_stack);

        self.stack = old_stack;

        value
    }

    // Roots result
    pub fn eval_expr(&mut self, expr: &syntax::Expr) -> Result {
        match expr {
            syntax::Expr::String(s) => Result::Ok(self.gc.rooted(Value::String(s.to_owned()))),
            syntax::Expr::Block(commands) => {
                // Not sure if rooting stack makes a difference here.
                
                let new_stack = self.gc.rooted(Stack {
                    frame: Frame { variables: HashMap::new() },
                    up: Some(self.stack)
                });

                let old_stack = self.stack;

                // I think unrooting is not needed?
                // self.gc.unroot(old_stack);
                self.stack = new_stack;

                let mut result = None;

                for command in &commands.0 {
                    if let Some(value) = result {
                        // Unused, I think.
                        self.gc.unroot(value);
                    }
                    result = Some(self.eval_cmd(&command)?);
                }

                self.gc.unroot(new_stack);
                // self.gc.root(old_stack);
                self.stack = old_stack;
                
                Ok(result.unwrap())
            }
            Expr::Closure(commands) => {
                Ok(self.gc.rooted(Value::Closure {
                    code: commands.clone(),
                    stack: self.stack,
                }))
            }
        }
    }
}

pub type Result<T = Gc<Value>, E = Vec<String>> = std::result::Result<T, E>;
