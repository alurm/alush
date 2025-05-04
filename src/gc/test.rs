use std::collections::VecDeque;

use super::Gc;
use super::Ptr;

#[test]
fn basic() {
    use super::tracing as gc;

    enum Tree<T> {
        Leaf(T),
        Branch {
            left: gc::Ptr<Tree<T>>,
            right: gc::Ptr<Tree<T>>,
        }
    }

    impl<T> Tree<T> {
        fn trace(&self) -> Vec<gc::Ptr<Self>> {
            match self {
                Self::Leaf(_) => Vec::new(),
                Self::Branch { left, right } => Vec::from([*left, *right]),
            }
        }
    }

    let mut gc = gc::Gc::new(Tree::trace);
    let hi = gc.alloc(Tree::Leaf("hi"));
    gc.root(hi);
    let world = gc.alloc(Tree::Leaf("world"));
    gc.root(world);
    let hi_world = gc.alloc(Tree::Branch { left: hi, right: world });
    gc.root(hi_world);

    fn print<'a>(gc: &gc::Gc<Tree<&'a str>>, message: gc::Ptr<Tree<&'a str>>) {
        let it = gc.get(message);

        match it {
            Tree::Leaf(m) => print!("{m}"),
            Tree::Branch { left, right } => {
                print(gc, *left);
                print!(" ");
                print(gc, *right);
            },
        }
    }

    print(&gc, hi_world);
    print!("\n");

    print(&gc, hi_world);
    print!("\n");

    gc.collect();
}

#[test]
fn mutate() {
    use super::tracing as gc;
    enum Tree<T> {
        Leaf(T),
        Branch {
            left: gc::Ptr<Tree<T>>,
            right: gc::Ptr<Tree<T>>,
        }
    }

    impl<T> Tree<T> {
        fn trace(&self) -> Vec<gc::Ptr<Self>> {
            match self {
                Self::Leaf(_) => Vec::new(),
                Self::Branch { left, right } => Vec::from([*left, *right]),
            }
        }
    }

    let mut gc = gc::Gc::new(Tree::trace);
    let a = gc.alloc(Tree::Leaf(1));
    gc.root(a);
    let b = gc.alloc(Tree::Leaf(2));
    gc.root(b);
    let c = gc.alloc(Tree::Branch { left: a, right: b });
    gc.root(c);

    fn recurse(gc: &mut gc::Gc<Tree<i32>>, ptr: gc::Ptr<Tree<i32>>) {
        let mut queue = VecDeque::<gc::Ptr<Tree<i32>>>::new();
        queue.push_back(ptr);
        while let Some(ptr) = queue.pop_front() {
            match gc.get_mut(ptr) {
                Tree::Leaf(value) => {
                    *value *= 2;
                    dbg!(value);
                },
                Tree::Branch { left, right } => {
                    queue.push_back(*left);
                    queue.push_back(*right);
                }
            }
        }
    }

    recurse(&mut gc, c);
}

#[should_panic]
#[test]
fn panic_after_unreachable_gced() {
    let mut gc = Gc::new(|_| Vec::new());

    let ptr = gc.alloc(0);

    gc.collect();

    gc.get(ptr);
}

#[test]
fn should_collect_cycle_and_terminate() {
    #[derive(Debug)]
    struct Cyclic(Option<Ptr<Cyclic>>);

    fn trace(it: &Cyclic) -> Vec<Ptr<Cyclic>> {
        if let Some(ptr) = it.0 {
            Vec::from([ptr])
        } else {
            Vec::new()
        }
    }

    let mut gc = Gc::new(trace);

    let cyclic = gc.alloc(Cyclic(None));

    gc.get_mut(cyclic).0 = Some(cyclic);

    dbg!(gc.get(cyclic));

    gc.root(cyclic);

    gc.collect();
}