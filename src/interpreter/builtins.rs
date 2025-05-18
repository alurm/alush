use std::collections::HashMap;

use gc::Gc;

use crate::{
    interpreter::{self, Env, Result, Value},
    syntax::Expr,
};

pub(crate) fn fail(_env: &mut Env, _args: &[Gc<Value>]) -> Result {
    Err(vec!["fail".into()])
}

pub(crate) fn apply(env: &mut Env, args: &[Gc<Value>]) -> Result {
    // Root once more since they'll be unrooted twice.
    args.iter().for_each(|&arg| {
        env.gc.root(arg);
    });
    let [fun, args @ ..] = args else {
        return Err(vec!["apply <fn> <args>...".into()]);
    };
    env.apply_cmd(*fun, args)
}

pub(crate) fn unix(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let args: Vec<String> = args
        .iter()
        .map(|&arg| {
            let Value::String(s) = env.gc.get(arg) else {
                return Err(vec!["unix <string>...".into()]);
            };
            Ok(s.to_owned())
        })
        .collect::<Result<_>>()?;
    let [head, rest @ ..] = &args[..] else {
        return Err(vec!["unix cmd <string>...".into()]);
    };
    let mut command = std::process::Command::new(head);
    command.stdout(std::process::Stdio::piped());
    command.args(rest);
    let process = command
        .spawn()
        .map_err(|err| vec!["unix: spawn:".into(), err.to_string()])?;
    let output = process
        .wait_with_output()
        .map_err(|err| vec!["unix: wait:".into(), err.to_string()])?;
    let string =
        String::from_utf8(output.stdout).map_err(|_| vec!["unix: output is not UTF-8".into()])?;
    Ok(env.gc.rooted(Value::String(string)))
}

pub(crate) fn lines(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [arg] = args else {
        return Err(vec!["lines <string>".into()]);
    };
    let Value::String(s) = env.gc.get(*arg) else {
        return Err(vec!["lines <string>".into()]);
    };
    let owned = s.to_owned();
    let mut map = HashMap::new();
    for (index, segment) in owned.split_terminator('\n').enumerate() {
        let key = env.gc.rooted(Value::String(index.to_string()));
        let value = env.gc.rooted(Value::String(segment.to_owned()));
        map.insert(key, value);
    }
    let map_value = env.gc.rooted(Value::Map(map));
    let Value::Map(map) = env.gc.get(map_value) else {
        unreachable!()
    };
    let entries = map.values().chain(map.keys()).copied().collect::<Vec<_>>();
    for entry in entries {
        env.gc.unroot(entry);
    }
    Ok(map_value)
}

// General continuations seem to be unimplementable as of now, not enough reification.
pub(crate) fn catch(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["catch <body>".into()]);
    };

    let value = env.eval_expr(body)?;

    if let Value::Exception(throw) = env.gc.get(value) {
        let throw = *throw;
        env.gc.root(throw);
        env.gc.unroot(value);
        Ok(throw)
    } else {
        Ok(value)
    }
}

pub(crate) fn throw(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let [throw] = args[..] else {
        return Err(vec!["throw <value>".into()]);
    };
    Ok(env.gc.rooted(Value::Exception(throw)))
}

pub(crate) fn concat(env: &mut Env, args: &[Gc<Value>]) -> Result {
    let mut result = String::new();
    for &arg in args {
        let Value::String(s) = env.gc.get(arg) else {
            return Err(vec!["..: <value: string>...".into()]);
        };
        result.push_str(s);
    }
    Ok(env.gc.rooted(Value::String(result)))
}

pub(crate) fn cond(env: &mut Env, args: &[Gc<Value>]) -> Result {
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

pub(crate) fn equal(env: &mut Env, args: &[Gc<Value>]) -> Result {
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

pub(crate) fn not_equal(env: &mut Env, args: &[Gc<Value>]) -> Result {
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

pub(crate) fn set(env: &mut Env, args: &[Gc<Value>]) -> Result {
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

pub(crate) fn val(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["val <value>".into()]);
    };
    env.gc.root(value);
    Ok(value)
}

pub(crate) fn println(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    for value in tail_values {
        let Value::String(value) = env.gc.get(*value) else {
            return Err(vec!["println <:string>...".into()]);
        };

        println!("{value}");
    }

    Ok(env.gc.rooted(Value::String("ok".into())))
}

pub(crate) fn var(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
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

pub(crate) fn get(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
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

pub(crate) fn del(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
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

pub(crate) fn inc(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let [value] = tail_values[..] else {
        return Err(vec!["inc <number>".into()]);
    };
    let Value::String(value) = env.gc.get(value) else {
        return Err(vec!["inc <number: string>".into()]);
    };
    let Ok(n) = value.parse::<isize>() else {
        return Err(vec!["inc: parse failed".into()]);
    };
    let str = format!("{}", n + 1);
    Ok(env.gc.rooted(Value::String(str)))
}

pub(crate) fn add(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let mut sum = 0;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else {
            return Err(vec!["+ <number: string>...".into()]);
        };
        let Ok(number) = value.parse::<isize>() else {
            return Err(vec!["+: parse failed".into()]);
        };
        sum += number;
    }
    let string = format!("{sum}");
    Ok(env.gc.rooted(Value::String(string)))
}

pub(crate) fn mul(env: &mut Env, tail_values: &[Gc<Value>]) -> Result {
    let mut product = 1;
    for &value in tail_values {
        let value = env.gc.get(value);
        let Value::String(value) = value else {
            return Err(vec!["* <number: string>...".into()]);
        };
        let Ok(number) = value.parse::<isize>() else {
            return Err(vec!["*: parse failed".into()]);
        };
        product *= number;
    }
    let string = format!("{product}");
    Ok(env.gc.rooted(Value::String(string)))
}

pub(crate) fn map(env: &mut Env, mut tail: &[Gc<Value>]) -> Result {
    let mut map = HashMap::new();
    while let [k, v, rest @ ..] = tail {
        map.insert(*k, *v);
        tail = rest;
    }
    Ok(env.gc.rooted(Value::Map(map)))
}

pub(crate) fn repeat(env: &mut Env, args: &[Expr]) -> Result {
    let [ref body] = args[..] else {
        return Err(vec!["repeat <body>".into()]);
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
