# To-do

- Make errors cause unroots to happen? RootGuard probably wouldn't work, but maybe it can wrap Env or Heap?
- Exceptions should be in Err, to avoid accidentally storing them anywhere.
- Make maps accept any `Gc<T>` as a key, not only strings?
- Make all examples tested.
- Add macros. For example, `inc x` can desugar to `set x $(+ $x 1)`.
- Add continuations.
- Make locals be looked up faster.
- Don't require ^D.
- Add gc::Heap::{enable, disable}?
- Pretty print $x as $x and not $('get' 'x'). Pretty print x as x and not 'x'
- Make builtins carry their name with them.
- Add `cd`.
- Hard exceptions seem to be overused. For example, `unix` shound't cause them, should it?
