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
    Lazy(fn(&mut Env, args: &[syntax::Expr]) -> Result),
    Throw(Gc<Value>),
}

impl gc::Trace for Value {
    fn trace(&self) -> Vec<gc::Id> {
        match self {
            Value::String(_) => Vec::new(),
            Value::Builtin(_) => Vec::new(),
            Value::Closure { code: _, stack } => vec![stack.id],
            Value::Throw(v) => vec![v.id],
            // I think this is correct.
            Value::Lazy(_) => Vec::new(),
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

// impl std::fmt::Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::Builtin(_) => f.write_str("<builtin>"),
//             Value::Closure { code: _, stack: _ } => f.write_str("<closure>"),
//             Value::String(s) => {
//                 let e = syntax::Expr::String(s.into());
//                 let mut pretty = String::new();
//                 e.pretty(&mut pretty, 0);
//                 f.write_str(&pretty)
//             }
//             Value::Throw(throw) => {
                
//             }
//         }
//     }
// }

fn lazy_loop(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["loop <body>".into()]);
    };
    loop {
        let value = env.eval_expr(body)?;
        if let Value::Throw(_) = env.gc.get(value) {
            return Ok(value)
        } else {
            env.gc.unroot(value);
        }
    }
}

/*
General continuations seem to be unimplementable as of now, not enough reification.
*/
fn lazy_catch(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["catch <body>".into()]);
    };

    let value = env.eval_expr(body)?;

    if let Value::Throw(throw) = env.gc.get(value) {
        // May be correct.
        let throw = *throw;
        env.gc.root(throw);
        env.gc.unroot(value);
        Ok(throw)
    } else {
        Ok(value)
    }
}

fn builtin_throw(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [throw] = args[..] else {
        return Err(vec!["throw <value>".into()]);
    };
    Ok(env.gc.rooted(Value::Throw(throw)))
}

fn builtin_concat(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let mut result = String::new();
    for &arg in args {
        let Value::String(s) = env.gc.get(arg) else {
            return Err(vec!["..: <value: string>...".into()]);
        };
        result.extend(s.chars());
    }
    Ok(env.gc.rooted(Value::String(result)))
}

fn builtin_if(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [cond, then, otherwise] = args[..] else {
        return Err(vec!["if <cond> <then> <else>".into()]);
    };
    let Value::String(cond) = env.gc.get(cond) else {
        return Err(vec!["if: <cond: string>".into()]);
    };
    if *cond == "true" {
        env.gc.root(then);
        Ok(then)
    } else {
        env.gc.root(otherwise);
        Ok(otherwise)
    }
}

fn builtin_eq(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [l, r] = args[..] else {
        return Err(vec!["= <a> <b>".into()]);
    };
    let (
        Value::String(l),
        Value::String(r)
    ) = (env.gc.get(l), env.gc.get(r)) else {
        return Ok(env.gc.rooted(Value::String("false".into())));
    };
    if l == r {
        Ok(env.gc.rooted(Value::String("true".into())))
    } else {
        Ok(env.gc.rooted(Value::String("false".into())))
    }
}

fn builtin_not_eq(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [l, r] = args[..] else {
        return Err(vec!["= <a> <b>".into()]);
    };
    let (
        Value::String(l),
        Value::String(r)
    ) = (env.gc.get(l), env.gc.get(r)) else {
        return Ok(env.gc.rooted(Value::String("true".into())));
    };
    if l == r {
        Ok(env.gc.rooted(Value::String("false".into())))
    } else {
        Ok(env.gc.rooted(Value::String("true".into())))
    }
}

fn builtin_set(env: &mut Env, args: &Vec<Gc<Value>>) -> Result {
    let [name, value] = args[..] else {
        return Err(vec!["set <name> <value>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["set <name: string> <value>".into()])
    };
    env.update(&name.to_owned(), value);
    Ok(env.gc.rooted(Value::String("ok".into())))
}

fn builtin_val(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["val <value>".into()]);
    };
    env.gc.root(value);
    Ok(value)
}

fn builtin_println(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    for value in tail_values {
        let Value::String(value) = env.gc.get(*value) else {
            return Err(vec!["println <:string>...".into()])
        };

        println!("{value}");
    }

    Ok(env.gc.rooted(Value::String("ok".into())))
}

fn builtin_var(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    for chunk in tail_values.chunks(2) {
        let [name, value] = chunk else {
            return Err(vec!["var { <name> <value> }".into()]);
        };
        
        let Value::String(name) = env.gc.get(*name) else {
            return Err(vec!["set <name: string> <value>".into()])
        };

        let name = name.into();

        let stack = env.gc.get_mut(env.stack);

        stack.frame.variables.insert(name, *value);
    }

    Ok(env.gc.rooted(Value::String("ok".into())))
}

fn builtin_get(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["get <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["get <name: string>".into()]); };
    let name = name.to_owned();
    let Some(value) = env.lookup(&name) else {
        return Err(vec!["get: var not found".into()]);
    };
    env.gc.root(value);
    Ok(value)
}

fn builtin_del(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let [name] = tail_values[..] else { return Err(vec!["del <name>".into()]) };
    let Value::String(name) = env.gc.get(name) else { return Err(vec!["del <name: string>".into()]); };
    let name = name.to_owned();
    if !env.forget(&name) {
        return Err(vec!["del: var not found".into()]);
    };
    Ok(env.gc.rooted(Value::String("ok".into())))
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
    // Ok(env.gc.alloc(Value::String("ok".into())))
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

fn builtin_mul(env: &mut Env, tail_values: &Vec<Gc<Value>>) -> Result {
    let mut product = 1;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else { return Err(vec!["add <number: string>...".into()])};
        let Ok(number) = value.parse::<i32>() else {
            return Err(vec!["add: parse failed".into()]);
        };
        product *= number;
    };
    let string = format!("{product}");
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
            ("+", builtin_add),
            ("*", builtin_mul),
            ("if", builtin_if),
            ("=", builtin_eq),
            ("..", builtin_concat),
            ("!=", builtin_not_eq),
            ("throw", builtin_throw),
            ("println", builtin_println),
        ];

        let builtins = builtins
            .map(|(k, v)| {
                (k.into(), gc.rooted(Value::Builtin(v)))
            });

        // Annoying clone here.
        let mut variables = HashMap::from(builtins.clone());

        let lazy_loop = gc.rooted(Value::Lazy(lazy_loop));
        variables.insert("loop".into(), lazy_loop);
        variables.insert("catch".into(), gc.rooted(Value::Lazy(lazy_catch)));

        // variables.insert("loop", );

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

        if let Value::Lazy(l) = self.gc.get(head) {
            let result = l(self, tail);
            self.gc.unroot(head);
            return result
        }

        let mut tail_values = Vec::new();

        for e in tail {
            // all rooted, need to unroot.
            // want to be lazier?
            
            let v = self.eval_expr(e)?;

            if let Value::Throw(_) = self.gc.get(v) {
                for value in tail_values {
                    self.gc.unroot(value);
                }
                return Ok(v);
            }

            tail_values.push(v);
        }

        match self.gc.get(head) {
            // errors should ideally cause unroots to happen?
            Value::String(_) => Err(vec!["cmd's fn must not be a string".into()]),
            Value::Builtin(f) => {
                let result = f(self, &tail_values);
                // unroot here on in builtins?
                for e in tail_values {
                    self.gc.unroot(e);
                }
                self.gc.unroot(head);
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
                self.gc.unroot(head);
                result
            }
            // _ => todo!()
            // Idk if this is correct.
            Value::Throw(_) => {
                // let value = *value;
                for e in tail_values {
                    self.gc.unroot(e);                    
                }
                Ok(head)
            }
            Value::Lazy(_) => panic!(),
        }
    }

    // No idea if this is correct.
    fn eval_closure(&mut self, commands: Rc<syntax::Commands>, stack: Gc<Stack>, args: &Vec<Gc<Value>>) -> Result {    
        let old_stack = self.stack;

        // I think stack should be reachable via closure so it should be fine to use it.
    
        // Append args as new stack frame.

        // Probably doesn't need to be rooted?
        // Well, maybe # should be? But isn't it reachable from the closure?
        let new_stack = self.gc.rooted(Stack {
            frame: Frame { variables: HashMap::new() },
            up: Some(stack),
        });

        let len = format!("{}", args.len());
        let len = self.gc.alloc(Value::String(len));

        let new_stack_value = self.gc.get_mut(new_stack);

        new_stack_value.frame.variables.insert("#".into(), len);
        
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
                
                if let Some(value) = result {
                    Ok(value)
                } else {
                    Ok(self.gc.rooted(Value::String("ok".into())))
                }
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
