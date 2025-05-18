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
- Make maps accept any `Gc<T>` as a key, not only strings?
- Make examples tested.
- Add macros.
- Add continuations.
- Make locals be looked up faster.
- Don't require ^D.
