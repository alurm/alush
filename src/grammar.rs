pub type Commands = Vec<Command>;
pub type Command = Vec<Expr>;

use crate::syntax::Input;

fn peek(i: &mut Input) -> Option<char> {
    i.peek().map(|&u| u)
}

fn accept(i: &mut Input, b: char) -> bool {
    if peek(i) == Some(b) {
        i.next();
        return true;
    }
    return false;
}

fn expect(i: &mut Input, b: char) -> Option<()> {
    if accept(i, b) {
        Some(())
    } else {
        None
    }
}

fn not(i: &mut Input, bs: &str) -> bool {
    if let Some(b) = peek(i) {
        for b2 in bs.chars() {
            if b == b2 {
                return false;
            }
        }
        i.next();
        return true;
    }
    return false;
}

#[derive(Debug)]
pub enum Expr {
    String {
        dollar: bool,
        value: String,
    },
    Commands {
        dollar: bool,
        value: Commands,
    }
}

fn string(i: &mut Input) -> Option<String> {
    if accept(i, '\'') {
        return Some(quoted_string(i)?);
    } else {
        let mut s = String::new();

        loop {
            match peek(i) {
                None | Some(' ' | '\n' | ')' | '\t') => {
                    if s.len() == 0 {
                        // Bad start of string.
                        return None;
                    } else {
                        return Some(s);
                    }
                },
                Some(b) => {
                    i.next();
                    s.push(b);
                },
            }    
        }
    }
}

fn quoted_string(i: &mut Input) -> Option<String> {
    let mut s = String::new();

    loop {
        if accept(i, '\'') {
            if accept(i, '\'') {
                s.push('\'');
            } else {
                return Some(s);
            }
        } else {
            if let Some(b) = i.next() {
                s.push(b);
            } else {
                // Unclosed quoted string.
                return None;
            }
        }
    }
}

fn expr(i: &mut Input) -> Option<Expr> {
    let dollar = accept(i, '$');

    if accept(i, '(') {
        return Some(Expr::Commands { dollar: dollar, value: commands(i)? });
    } else {
        return Some(Expr::String { dollar: dollar, value: string(i)? });
    }
}

fn commands(i: &mut Input) -> Option<Commands> {
    if accept(i, '\n') {
        return multiline_commands(i);
    } else {
        return inline_command(i);
    }
}

// // Interestingly using shell makes trailing comments not allowed.
// pub fn file(i: &mut Input) -> Option<Commands> {
//     let mut commands = Commands::new();
//     loop {
//         if let None = peek(i) {
//             return Some(commands)
//         }
//         commands.push(shell(i)?)
//     }
// }

pub fn file(i: &mut Input) -> Option<Commands> {
    let mut commands = Commands::new();
    loop {
        loop {
            match peek(i) {
                Some(' ') | Some('\t') => {
                    i.next();
                },
                _ => break,
            }
        }
        if let None = peek(i) {
            return Some(commands)
        }
        else if accept(i, '#') { comment(i)? }
        else if accept(i, '\n') {}
        else {
            commands.push(command(i)?);
            expect(i, '\n')?;
        }
    }
}

pub fn multiline_commands(i: &mut Input) -> Option<Commands> {
    let mut commands = Commands::new();
    loop {
        loop {
            match peek(i) {
                Some(' ') | Some('\t') => {
                    i.next();
                },
                _ => break,
            }
        }
        if accept(i, ')') {
            return Some(commands);
        }
        else if accept(i, '#') { comment(i)? }
        else if accept(i, '\n') {}
        else {
            commands.push(command(i)?);
            expect(i, '\n')?;
        }
    }
}

fn comment(i: &mut Input) -> Option<()> {
    while not(i, "\n") {}
    expect(i, '\n')
}

// Initially adapted from multiline_commands().
pub fn shell(i: &mut Input) -> Option<Command> {
    loop {
        while let Some(' ' | '\t') = peek(i) {
            i.next();
        }
        if accept(i, '#') { comment(i)? }
        else if accept(i, '\n') {}
        else {
            let c = command(i)?;
            expect(i, '\n')?;
            return Some(c);
        }
    }
}

fn inline_command(i: &mut Input) -> Option<Commands> {
    if accept(i, ')') {
        return Some(Commands::new());
    } else {
        let commands = vec![command(i)?];

        expect(i, ')')?;

        return Some(commands);
    }
}

fn multiline_command_part(i: &mut Input) -> Option<Vec<Expr>> {
    let mut exprs = Vec::new();
    loop {
        while let Some(' ' | '\t') = peek(i) { i.next(); }
        if accept(i, ';') {
            return Some(exprs);
        } else if accept(i, '#') {
            comment(i);
        } else if accept(i, '\n') {}
        // We could've called back to command here to allow for recursion.
        // But it's not clear that it's better.
        else {
            exprs.push(expr(i)?);

            while accept(i, ' ') {
                exprs.push(expr(i)?);
            }
        }
    }
}

pub fn command(i: &mut Input) -> Option<Command> {
    let mut command = Command::new();

    command.push(expr(i)?);

    while accept(i, ' ') {
        if accept(i, '\\') {
            let mut exprs = multiline_command_part(i)?;
            command.append(&mut exprs);
        } else {
            command.push(expr(i)?);
        }
    }

    return Some(command);
}
