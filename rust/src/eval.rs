use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::syntax;

#[derive(Clone)]
pub enum Value {
    String(String),
    // Builtin(fn(syntax::Command) -> Value),
}

pub struct EnvNode {
    pub up: Env,
    pub variables: HashMap<String, Value>,
}

pub type Env = Option<Rc<RefCell<EnvNode>>>;

pub trait Eval {
    fn eval(&self, env: Env) -> Value;
}

fn lookup(s: &str, mut env: Env) -> Option<Value> {
    match env {
        None => return None,
        Some(ref env) => {
            let env = env.borrow();
            if let Some(value) = env.variables.get(s) {
                return Some(value.clone());
            }
            return lookup(s, env.up.clone());
        }
    }
}

impl Eval for syntax::Command {
    fn eval(&self, env: Env) -> Value {
        let head = &self.0[0];
        let tail = &self.0[1..];

        match head {
            syntax::Expr::String(string) => {
                match string.as_str() {
                    "echo" => {
                        let mut sep = "";

                        for e in tail {
                            if let Value::String(s) = e.eval(env.clone()) {
                                print!("{sep}");
                                print!("{s}");
                            } else {
                                panic!("echo: arg is not str");
                            }
                            sep = " ";
                        }

                        print!("\n");

                        return Value::String("".to_string());
                    },
                    "get" => {
                        if let syntax::Expr::String(s) = &tail[0] {
                            if let Some(v) = lookup(s, env) {
                                return v;
                            } else {
                                panic!("get: no val for var");
                            }
                        } else {
                            panic!("get: arg is not str")
                        }
                    }
                    "var" => {
                        todo!()  
                    }
                    _ => todo!()
                }
            }
            _ => todo!()
        }    
    }
}

impl Eval for syntax::Expr {
    fn eval(&self, env: Env) -> Value {
        match self {
            syntax::Expr::String(string) => Value::String(string.clone()),
            syntax::Expr::Block(commands) => {
                assert!(commands.0.len() != 0);
                let mut value = None;
                for command in &commands.0 {
                    value = Some(command.eval(env.clone()));
                }
                value.unwrap()
            }
            _ => todo!()
        }
    }
}