# A Rust cycle collecting garbage collector and a shell interpreter with closures and maps

- [The garbage collector's README](./gc/README.md).

# The shell interpreter

The syntax is intentionally simple. At first sight, it looks most similar to a POSIX shell. [Grammar](./doc/syntax.md).

There are strings, commands, blocks, variables and closures.

## Commands

```shell
# This is a comment.
println 'The first argument.' the-second-argument
```

Commands span multiple lines. Multiline part of a command starts with a backslash followed by a newline and ends with a semicolon.

```shell
println \
    # A comment may be put in the middle of a multiline command.
    hello world
;
```

## Variables

```shell
# Let's make a variable "count" with the value of zero.
var count 0
# Let's use the variable.
println $count
# In fact, $foo is a syntactic sugar for $(get foo).
# So, we could've written the previous command as so:
println $(get count)
```

## Closures

```shell
(
    # This is a closure in the function position of a command.
    # Thus, it will be executed immediately.
    # However, it can contain local variables.
    var greeting 'Hello'
    println $greeting
)
# Greeting is no longer available.
```

## Blocks

```shell
println $(
    # This is a block.
    # A block's value is the value of the last command in the block.
    # Block can have its own local variables.
    var count 0
    # `..` is the function to concatenate strings together.
    .. 'count: ' $count
)
```

## Builtins

```shell
# The map of all builtins can be acquired by the "vars" command.
vars
# You may notice that some builitns are "lazy" ("catch", "if", "repeat", "assert", to name a few).
# That means that they are like macros: they take arguments as expressions.
# That's why "if" doesn't actually evaluate the else branch "$(fail)".
if true $(println cool) $(fail)
```

## Maps

Keys of maps have to be strings. Maps preserve their insertion order.

```shell
# We can use the map builtin to create a map.
var m $(
    map \
        # The key is a, the value is b.
        a b
        # The key is c, the value is another map.
        c $(map d e)
    ;
)
# Here's some "methods" of maps.
assert $(m has a)
assert $(= b $(m get a))
m del a
assert $(= false $(m has a))
m set k v
assert $(= $(m get k) v)

# The "unix" function may be used to call out to external programs.
var content $(unix cat Cargo.toml)
var by-lines $(lines $content)
# Prints the first line.
println $(by-lines get 0)
```

## More examples

[Take a look at examples](./examples). You may also take a look at the [tests](./src/tests.rs).

## To-do

Some things to-do are outlined [here](./doc/ideas.md).
