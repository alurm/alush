```
expr = [ '$' ] ( '(' commands | string ) ;

string =
    | '\'' quoted-string
    | not(' ', '\n', ')', '\t') { not(' ', '\n', ')') }
;

quoted-string =
    | '\'' [ '\'' quoted-string ]
    | any() quoted-string
;

commands =
    | '\n' multiline-commands
    | inline-command
;

command = expr { ' ' expr } ;

inline-command = ')' | command ')' ;

multiline-commands = { ( ' ' | '\t' ) } (
    | ')'
    | '#' { not('\n') } '\n' multiline-commands
    | '\n' multiline-commands
    | command '\n' multiline-commands
) ;
```

# Example

```text
var make-adder (
    var sum 0
    put (
        set sum $(+ $sum $0)
        get sum
    )
)

var adder $(make-adder)

(
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
