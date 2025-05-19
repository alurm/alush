# Approximate grammar

```
expr = [ '$' ] ( '(' commands | string ) ;

string =
    | '\'' quoted-string
    | not(' ', '\n', ')', '\t') { not(' ', '\n', ')') }
;

any = not()

quoted-string =
    | '\'' [ '\'' quoted-string ]
    | any quoted-string
;

commands =
    | '\n' multiline-commands
    | inline-command
;

multiline-command-part = { ' ' | '\t' } (
    | ';'
    | '#' comment multiline-command-part
    | '\n' multiline-command-part
    | expr { ' ' expr } '\n' multiline-command-part
)

comment = { not('\n') } '\n' ;

command = expr [
    ' ' (
        | '\\' multiline-command-part command
        | expr command
    )
] ;

inline-command = ')' | command ')' ;

file = { ' ' | '\t' } (
    | ''
    | '#' comment file
    | '\n' file
    | command '\n' file
);

interactive = { ' ' | '\t' } (
    | '#' comment interactive
    | '\n' interactive
    | command '\n' interactive
)

multiline-commands = { ' ' | '\t' } (
    | ')'
    | '#' comment multiline-commands
    | '\n' multiline-commands
    | command '\n' multiline-commands
) ;
```
