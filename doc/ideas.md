# Implementation language

- C: no generics.
- C++: too complex.
- Go: no tagged unions.
- OCaml: the build system is too complex.
- Java: bloated.
- Zig: not stable, no type inference for generics.
- D: unpopular, complex.
- Lua: slow.
- Rust: too complex.

# Features

- A GC.
- Closures.

# [Syntax](./syntax.md)

# To-do

- Make errors cause unroots to happen?
- Exceptions should be in Err, to avoid accidentally storing them anywhere.
- Make maps accept any `Gc<T>` as a key, not only strings?
- Make examples tested.
- Add macros.
- Add continuations.
- Make locals be looked up faster.
- Don't require ^D.
- Add gc::Heap::{enable, disable}?
- Pretty print $x as $x and not $('get' 'x'). Pretty print x as x and not 'x'
- Make builtins carry their name with them.
