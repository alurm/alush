use std::{collections::HashMap, rc::Rc};

use gc::{self, Gc};

use crate::syntax::{self, Expr};

mod builtins;

// pub use builtins::*;

pub enum Callable {
    Closure {
        code: Rc<syntax::Commands>,
        stack: Gc<Stack>,
    },
}

type Builtin = fn(&mut Env, args: &[Gc<Value>]) -> Result;
type LazyBuiltin = fn(&mut Env, args: &[syntax::Expr]) -> Result;

pub enum Value {
    String(String),
    Builtin(Builtin),
    Callable(Callable),
    LazyBuiltin(LazyBuiltin),
    Exception(Gc<Value>),
    // Map(HashMap<Gc<Value>, Gc<Value>>),
    Map(HashMap<String, Gc<Value>>),
}

impl gc::Collect for Value {
    fn trace(&self) -> Vec<gc::Id> {
        match self {
            Value::String(_) => Vec::new(),
            Value::Builtin(_) => Vec::new(),
            Value::Callable(Callable::Closure { code: _, stack }) => vec![stack.id],
            Value::Exception(v) => vec![v.id],
            Value::LazyBuiltin(_) => Vec::new(),
            Value::Map(m) => {
                let mut result = Vec::new();
                for (_, &v) in m {
                    result.push(v.id);
                }
                result
            }
        }
    }
}

pub struct Frame {
    pub variables: HashMap<String, Gc<Value>>,
}

pub struct Stack {
    pub frame: Frame,
    pub up: Option<Gc<Stack>>,
}

impl gc::Collect for Stack {
    fn trace(&self) -> Vec<gc::Id> {
        let mut vec = self.frame.trace();
        if let Some(up) = self.up {
            vec.push(up.id);
        }
        vec
    }
}

// pub struct Stack(pub Vec<Frame>);

impl gc::Collect for Frame {
    fn trace(&self) -> Vec<gc::Id> {
        let mut vec = Vec::new();
        for v in self.variables.values() {
            vec.push(v.id);
        }
        vec
    }
}

pub struct Env {
    pub gc: gc::Heap,
    pub stack: Gc<Stack>,
    // strings: Strings,
}

impl Env {
    pub fn new(strategy: gc::Strategy) -> Self {
        let mut gc = gc::Heap::new(strategy);

        let builtins: &[(_, Builtin)] = &[
            ("set", builtins::set),
            ("get", builtins::get),
            ("val", builtins::val),
            ("var", builtins::var),
            ("del", builtins::del),
            ("inc", builtins::inc),
            ("+", builtins::add),
            ("*", builtins::mul),
            ("=", builtins::equal),
            ("..", builtins::concat),
            ("!=", builtins::not_equal),
            ("throw", builtins::throw),
            ("println", builtins::println),
            ("map", builtins::map),
            ("fail", builtins::fail),
            ("apply", builtins::apply),
            ("unix", builtins::unix),
            ("lines", builtins::lines),
        ];

        let lazy_builtins: &[(_, LazyBuiltin)] = &[
            ("repeat", builtins::repeat),
            ("catch", builtins::catch),
            ("if", builtins::cond),
        ];

        let mut variables = Vec::new();

        variables.extend(
            builtins
                .iter()
                .map(|(k, v)| ((*k).into(), gc.rooted(Value::Builtin(*v)))),
        );

        variables.extend(
            lazy_builtins
                .iter()
                .map(|(k, v)| ((*k).into(), gc.rooted(Value::LazyBuiltin(*v)))),
        );

        let variables = HashMap::from_iter(variables);

        let stack = gc.rooted(Stack {
            frame: Frame { variables },
            up: None,
        });

        gc
            .get(stack)
            .frame
            .variables
            .values()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|var| gc.unroot(var));

        Env { gc, stack }
    }

    pub fn lookup(&mut self, name: &str) -> Option<Gc<Value>> {
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

    pub fn update(&mut self, name: &str, value: Gc<Value>) -> bool {
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

    pub fn forget(&mut self, name: &str) -> bool {
        let mut maybe_stack = Some(self.stack);
        while let Some(stack) = maybe_stack {
            let stack = self.gc.get_mut(stack);
            if stack.frame.variables.remove(name).is_some() {
                return true;
            }
            maybe_stack = stack.up;
        }
        false
    }

    fn apply_cmd(&mut self, head: Gc<Value>, tail_values: &[Gc<Value>]) -> Result {
        match self.gc.get(head) {
            Value::Map(map) => {
                /*
                    map get k
                    map set k v
                    map del k
                    map has k
                    map each fun
                */
                let [command, ref rest @ ..] = tail_values[..] else {
                    return Err(vec!["map: <command> ...".into()]);
                };
                let Value::String(command) = self.gc.get(command) else {
                    return Err(vec!["map: <command: string> ...".into()]);
                };
                match command.as_str() {
                    "each" => {
                        let [fun] = rest else {
                            return Err(vec!["map each <fn>".into()]);
                        };
                        // Cloning is expensive...
                        let mut result = None;
                        for (k, v) in map.clone() {
                            if let Some(result) = result {
                                self.gc.unroot(result);
                            }
                            let args = vec![self.gc.rooted(Value::String(k)), self.gc.root(v)];
                            self.gc.root(*fun);
                            result = Some(self.apply_cmd(*fun, &args)?);
                        }
                        // // let command = syntax::Command(Vec::new());
                        // // command.
                        // // self.eval_cmd()
                        // // for (k, v) in map {
                        // //     call()
                        // // }
                        self.gc.unroot(head);
                        for &v in tail_values {
                            self.gc.unroot(v);
                        }
                        Ok(result.unwrap_or_else(|| self.gc.rooted(Value::String("ok".into()))))
                    }
                    "get" => {
                        let [k] = rest else {
                            return Err(vec!["map get <key>".into()]);
                        };
                        let Value::String(k) = self.gc.get(*k) else {
                            return Err(vec!["map del <key: string>".into()]);
                        };
                        let Some(&v) = map.get(k) else {
                            return Err(vec!["map get: key not found".into()]);
                        };
                        self.gc.unroot(head);
                        for &r in tail_values {
                            self.gc.unroot(r);
                        }
                        // Need to root v because it's always done.
                        self.gc.root(v);
                        Ok(v)
                    }
                    "del" => {
                        let [k] = rest else {
                            return Err(vec!["map del <key>".into()]);
                        };
                        let Value::String(k) = self.gc.get(*k) else {
                            return Err(vec!["map del <key: string>".into()]);
                        };
                        let k = k.clone();
                        let Value::Map(map) = self.gc.get_mut(head) else {
                            unreachable!()
                        };
                        map.remove(&k);
                        self.gc.unroot(head);
                        for &r in tail_values {
                            self.gc.unroot(r);
                        }
                        Ok(self.gc.rooted(Value::String("ok".into())))
                    }
                    "has" => {
                        let [k] = rest else {
                            return Err(vec!["map has <key>".into()]);
                        };
                        let Value::String(k) = self.gc.get(*k) else {
                            return Err(vec!["map del <key: string>".into()]);
                        };
                        let has = map.contains_key(k);
                        self.gc.unroot(head);
                        for &r in tail_values {
                            self.gc.unroot(r)
                        }
                        Ok(self
                            .gc
                            .rooted(Value::String(if has { "true" } else { "false" }.into())))
                    }
                    "set" => {
                        let [k, v] = rest else {
                            return Err(vec!["map set <key> <value>".into()]);
                        };
                        let Value::String(k) = self.gc.get(*k) else {
                            return Err(vec!["map del <key: string>".into()]);
                        };
                        let k = k.clone();
                        let Value::Map(map) = self.gc.get_mut(head) else {
                            unreachable!()
                        };
                        map.insert(k, *v);
                        self.gc.unroot(head);
                        for &r in tail_values {
                            self.gc.unroot(r)
                        }
                        Ok(self.gc.rooted(Value::String("ok".into())))
                    }
                    _ => Err(vec!["map: unknown command".into()]),
                }
            }
            Value::String(_) => Err(vec!["cmd's fn must not be a string".into()]),
            Value::Builtin(f) => {
                let result = f(self, tail_values);
                // unroot here on in builtins?
                for &e in tail_values {
                    self.gc.unroot(e);
                }
                self.gc.unroot(head);
                result
            }
            // stack should be reachable via closure.
            Value::Callable(Callable::Closure { code, stack }) => {
                let result = self.eval_closure(code.clone(), *stack, tail_values);
                for &e in tail_values {
                    self.gc.unroot(e);
                }
                self.gc.unroot(head);
                result
            }
            // _ => todo!()
            // Idk if this is correct.
            Value::Exception(_) => {
                // let value = *value;
                for &e in tail_values {
                    self.gc.unroot(e);
                }
                Ok(head)
            }
            Value::LazyBuiltin(_) => panic!(),
        }
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
                let Some(value) = self.lookup(string) else {
                    return Err(vec!["lookup failed".into()]);
                };
                // need to root so doesn't disappear during eval? idk.
                self.gc.root(value);
                value
            }
            _ => self.eval_expr(head)?, // rooted, need to unroot later
        };

        if let Value::LazyBuiltin(l) = self.gc.get(head) {
            let result = l(self, tail);
            self.gc.unroot(head);
            return result;
        }

        let mut tail_values = Vec::new();

        for e in tail {
            // all rooted, need to unroot.
            // want to be lazier?

            let v = self.eval_expr(e)?;

            if let Value::Exception(_) = self.gc.get(v) {
                for value in tail_values {
                    self.gc.unroot(value);
                }
                self.gc.unroot(head);
                return Ok(v);
            }

            tail_values.push(v);
        }

        self.apply_cmd(head, &tail_values)
    }

    // No idea if this is correct.
    fn eval_closure(
        &mut self,
        commands: Rc<syntax::Commands>,
        stack: Gc<Stack>,
        args: &[Gc<Value>],
    ) -> Result {
        let closure_stack = self.stack;

        // Probably doesn't need to be rooted?
        // Well, maybe # should be? But isn't it reachable from the closure?
        let new_stack = self.gc.rooted(Stack {
            frame: Frame {
                variables: HashMap::new(),
            },
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

        self.stack = closure_stack;

        value
    }

    // Roots result
    pub fn eval_expr(&mut self, expr: &syntax::Expr) -> Result {
        match expr {
            syntax::Expr::String(s) => Result::Ok(self.gc.rooted(Value::String(s.to_owned()))),
            syntax::Expr::Block(commands) => {
                // Not sure if rooting stack makes a difference here.

                let new_stack = self.gc.rooted(Stack {
                    frame: Frame {
                        variables: HashMap::new(),
                    },
                    up: Some(self.stack),
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
                    result = Some(self.eval_cmd(command)?);
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
            Expr::Closure(commands) => Ok(self.gc.rooted(Value::Callable(Callable::Closure {
                code: commands.clone(),
                stack: self.stack,
            }))),
        }
    }
}

pub type Result<T = Gc<Value>, E = Vec<String>> = std::result::Result<T, E>;
