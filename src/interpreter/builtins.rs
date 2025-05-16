use std::collections::HashMap;

use gc::Gc;

use crate::{
    interpreter::{Env, Result, Value},
    syntax::Expr,
};

/*
General continuations seem to be unimplementable as of now, not enough reification.
*/
pub(crate) fn lazy_catch(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["catch <body>".into()]);
    };

    let value = env.eval_expr(body)?;

    if let Value::Exception(throw) = env.gc.get(value) {
        // May be correct.
        let throw = *throw;
        env.gc.root(throw);
        env.gc.unroot(value);
        Ok(throw)
    } else {
        Ok(value)
    }
}

pub(crate) fn builtin_throw(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [throw] = args[..] else {
        return Err(vec!["throw <value>".into()]);
    };
    Ok(env.gc.rooted(Value::Exception(throw)))
}

pub(crate) fn builtin_concat(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let mut result = String::new();
    for &arg in args {
        let Value::String(s) = env.gc.get(arg) else {
            return Err(vec!["..: <value: string>...".into()]);
        };
        result.push_str(s);
    }
    Ok(env.gc.rooted(Value::String(result)))
}

pub(crate) fn builtin_if(env: &mut Env, args: &[Gc<Value>]) -> Result {
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

pub(crate) fn builtin_eq(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [l, r] = args[..] else {
        return Err(vec!["= <a> <b>".into()]);
    };
    let (Value::String(l), Value::String(r)) = (env.gc.get(l), env.gc.get(r)) else {
        return Ok(env.gc.rooted(Value::String("false".into())));
    };
    if l == r {
        Ok(env.gc.rooted(Value::String("true".into())))
    } else {
        Ok(env.gc.rooted(Value::String("false".into())))
    }
}

pub(crate) fn builtin_not_eq(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [l, r] = args[..] else {
        return Err(vec!["= <a> <b>".into()]);
    };
    let (Value::String(l), Value::String(r)) = (env.gc.get(l), env.gc.get(r)) else {
        return Ok(env.gc.rooted(Value::String("true".into())));
    };
    if l == r {
        Ok(env.gc.rooted(Value::String("false".into())))
    } else {
        Ok(env.gc.rooted(Value::String("true".into())))
    }
}

pub(crate) fn builtin_set(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [name, value] = args[..] else {
        return Err(vec!["set <name> <value>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["set <name: string> <value>".into()]);
    };
    if !env.update(&name.to_owned(), value) {
        return Err(vec!["set: var not found".into()]);
    }
    Ok(env.gc.rooted(Value::String("ok".into())))
}

pub(crate) fn builtin_val(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["val <value>".into()]);
    };
    env.gc.root(value);
    Ok(value)
}

pub(crate) fn builtin_println(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    for value in tail_values {
        let Value::String(value) = env.gc.get(*value) else {
            return Err(vec!["println <:string>...".into()]);
        };

        println!("{value}");
    }

    Ok(env.gc.rooted(Value::String("ok".into())))
}

pub(crate) fn builtin_var(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    for chunk in tail_values.chunks(2) {
        let [name, value] = chunk else {
            return Err(vec!["var { <name> <value> }".into()]);
        };

        let Value::String(name) = env.gc.get(*name) else {
            return Err(vec!["set <name: string> <value>".into()]);
        };

        let name = name.into();

        let stack = env.gc.get_mut(env.stack);

        stack.frame.variables.insert(name, *value);
    }

    Ok(env.gc.rooted(Value::String("ok".into())))
}

pub(crate) fn builtin_get(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [name] = tail_values[..] else {
        return Err(vec!["get <name>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["get <name: string>".into()]);
    };
    let name = name.to_owned();
    let Some(value) = env.lookup(&name) else {
        return Err(vec!["get: var not found".into()]);
    };
    env.gc.root(value);
    Ok(value)
}

pub(crate) fn builtin_del(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [name] = tail_values[..] else {
        return Err(vec!["del <name>".into()]);
    };
    let Value::String(name) = env.gc.get(name) else {
        return Err(vec!["del <name: string>".into()]);
    };
    let name = name.to_owned();
    if !env.forget(&name) {
        return Err(vec!["del: var not found".into()]);
    };
    Ok(env.gc.rooted(Value::String("ok".into())))
}

pub(crate) fn builtin_inc(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["inc <number>".into()]);
    };
    let Value::String(value) = env.gc.get(value) else {
        return Err(vec!["inc <number: string>".into()]);
    };
    let Ok(n) = value.parse::<i32>() else {
        return Err(vec!["inc: parse failed".into()]);
    };
    let str = format!("{}", n + 1);
    Ok(env.gc.rooted(Value::String(str)))
    // let name = name.to_owned();
    // if !env.forget(&name) {
    //     return Err(vec!["del: var not found".into()]);
    // };
    // Ok(env.gc.alloc(Value::String("ok".into())))
}

pub(crate) fn builtin_add(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let mut sum = 0;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else {
            return Err(vec!["add <number: string>...".into()]);
        };
        let Ok(number) = value.parse::<i32>() else {
            return Err(vec!["add: parse failed".into()]);
        };
        sum += number;
    }
    let string = format!("{sum}");
    Ok(env.gc.rooted(Value::String(string)))
}

pub(crate) fn builtin_mul(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let mut product = 1;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else {
            return Err(vec!["add <number: string>...".into()]);
        };
        let Ok(number) = value.parse::<i32>() else {
            return Err(vec!["add: parse failed".into()]);
        };
        product *= number;
    }
    let string = format!("{product}");
    Ok(env.gc.rooted(Value::String(string)))
}

pub(crate) fn builtin_map(env: &mut Env, mut tail: &[Gc<Value>]) -> Result {
    let mut map = HashMap::new();
    while let [k, v, rest @ ..] = tail {
        map.insert(*k, *v);
        tail = rest;
    }
    Ok(env.gc.rooted(Value::Map(map)))
}

pub(crate) fn lazy_loop(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["loop <body>".into()]);
    };
    loop {
        let value = env.eval_expr(body)?;
        // It's annoying that we have to handle this here manually.
        if let Value::Exception(_) = env.gc.get(value) {
            return Ok(value);
        } else {
            env.gc.unroot(value);
        }
    }
}
