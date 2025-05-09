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

# Example

```text
var make-adder (
    var sum 0
    val (
        set sum $(+ $sum $1)
        get sum
    )
)
```

## Internal representation

```text
call(var,
    make-adder,
    closure(do,
        closure(???)
    )
    namespace(
        closure(
            do()
        )
    )
)
```

# Early returns

```text
var ok-3 (
    var type ok
    var value 3
    object type value
)

var \
    x
    y 4
;

ok-3 type
ok-3 value

ok-3 

var parse-number (
    var result $(<parse-number> $1)
)

var do-if (
    $(if $1 $2 $3)
)

var add (
    var return $(early-return)

    var string $(
        do-if $(= $(= $# 1) '') (
            early-return ''
        ) (
            get 1
        )
    )

    var number $(
        var parse $(parse-number $string)
        do-if $(= $(parse type) err) (early-return $(map type err))
        parse value
    )

    + $number $number

    var string $(if $(= $(= $# 1) '') (early-return $()) (get 1))

    var parse $(parse-number $string)

    if $(= $(parse type) err) (early-return)

    # ok number
    # bad
    (ok number)
    (bad foo)
    parse-number $string
    
    var $(args 0)
    
    var $(0)
    $args(0)
    $args(1)

    $(args 1)
    var input $1
)

var test-add (
    var number 3
    var bad ''
    var result:
        $(add $number $bad)
    var result $(add $number $bad)
)
```

# Example

```text
var make-adder (
    var sum 0
    val (
        set sum $(+ $sum $1)
        get sum
    )
)

var adder $(make-adder)

let come $(come-from)

if $(!= $come came)

catch (
    var throw $1

    var count 0

    loop (
        set count $(+ 1 $count)
        if $(= $count 10) (throw $count)
    )
)

catch (
    var return $1
    var count 0

    var s $(stack)


    loop (
        set count $(+ 1 $count)
        if $(= $count 10) (return $count)
    )
)

catch (
    var count 0
    loop (
        var return $0
        adder $count
        set count $(+ $count 1)
        if $(= $count 10) (return)
    )
)

var open (
    var path $0
    var mode $1
    var error-handler $2
    var handle $(native-open path mode)
    if $(= $handle -1) (error-handler) $handle
)

# catch (
#     var catch $0
#     var file $(open 'Joe''s path' rw $catch)
#     defer (close $file)
# )

# var sequence-generator $(
#     generator (
#         var yield $0
#         var number 0
#         loop (
#             yield $number
#             set number $(+ 1 $number)
#         )
#     )
# )

# var g $(sequence-generator)
# loop (
#     var return $0
#     var number $(g)
#     if $(= $number 10) (return)
#     print $number
# )
```
