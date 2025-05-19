# The cycle collecting garbage collector

## Heap

The system provides a `gc::Heap` via which one can make `Gc<T>`s by `.alloc`ing them, provided that `T: Collect`.

To create a set of roots, handles may be `.root`ed and `.unroot`ed.

## The "Collect" trait

`Collect` requires a type to implement `trace`, which will report objects reachable from the current one.

`Collect` is also `Any`. The same heap may host multiple different types.

## Garbage collection strategies

There are multiple GC strategies:

- Disabled (GC is not run)
- Default (GC will run once the amount of live objects doubles)
- Aggressive (GC will run on every allocation)
- Checking (GC will run on every allocation and a log will be printed. Underrooting causes a panic)
- `Heap::collect` may be used to manually cause garbage collection

## Safety

`Gc` contains a unique number which identifies the heap. Trying to use it to access a heap which didn't allocate the `Gc` will panic.

There are currently no finalizers or weak references.
