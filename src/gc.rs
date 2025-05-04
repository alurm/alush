/// A garbage collector.

// The idea in this iteration is to use std::any::Any.

use std::{
    any::{self, Any, TypeId},
    collections::{HashMap, HashSet, VecDeque},
    marker::PhantomData, ptr, sync::Mutex,
};

struct Object {
    value: Box<dyn Trace>,
    reachable: bool,
}

/// An opaque id for a value in the GC's heap.
/// Useful for implementing the [Trace] trait.
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Id {
    heap: usize,
    index: usize,
}

/// A handle to a value allocated in the GC's [Heap].
pub struct Gc<T: Trace> {
    pub id: Id,
    phantom_data: PhantomData<T>,
}

impl<T: Trace> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Trace> Copy for Gc<T> {}

/// A trait to trace though a GC'ed value.
pub trait Trace: Any {
    fn trace(&self) -> Vec<Id>;
}

/// An owner of [Gc]s.
pub struct Heap {
    map: HashMap<Id, Object>,
    roots: HashSet<Id>,
    counter: usize,
    capacity: usize,
    id: usize,
    strategy: Strategy,
}

pub enum Strategy {
    Default,
    Aggressive,
    Disabled,
}

static COUNTER: Mutex<usize> = Mutex::new(0);

impl Heap {
    pub fn new(strategy: Strategy) -> Self {
        Heap {
            capacity: 0,
            map: HashMap::new(),
            roots: HashSet::new(),
            counter: 0,
            strategy,
            id: {
                let mut guard = COUNTER.lock().unwrap();
                *guard = *guard + 1;
                *guard
            }
        }
    }

    pub fn alloc<T: Trace>(&mut self, init: T) -> Gc<T> {
        match self.strategy {
            Strategy::Aggressive => self.collect(),
            Strategy::Default if self.map.len() == self.capacity => self.collect(),
            Strategy::Default | Strategy::Disabled => {}
        }

        self.counter += 1;

        let id = Id {
            index: self.counter,
            heap: self.id,
        };

        let object = Object {
            value: Box::new(init),
            reachable: false,
        };

        self.map.insert(id, object);

        Gc { id, phantom_data: PhantomData }
    }

    fn get_object(&self, id: Id) -> &Object {
        assert!(self.id == id.heap);
        self.map.get(&id).unwrap()
    }

    fn get_mut_object(&mut self, id: Id) -> &mut Object {
        assert!(self.id == id.heap);
        self.map.get_mut(&id).unwrap()
    }

    /// Returns a shared reference to a value contained in a [Gc].
    pub fn get<T: Trace>(&self, id: Gc<T>) -> &T {
        let object = self.get_object(id.id);
        let it = &*object.value as &dyn Any;
        match it.downcast_ref::<T>() {
            None => panic!(),
            Some(r) => r,
        }
    }

    /// Returns a mutable reference to a value contained in a [Gc].
    pub fn get_mut<T: Trace>(&mut self, id: Gc<T>) -> &mut T {
        let object = self.get_mut_object(id.id);
        let it = &mut *object.value as &mut dyn Any;
        match it.downcast_mut::<T>() {
            None => panic!(),
            Some(r) => r,
        }
    }

    /// Prevents a [Gc] from being collected.
    pub fn root<T: Trace>(&mut self, id: Gc<T>) {
        self.roots.insert(id.id);
    }

    /// Allows a [Gc] to be collected, if discovered to be unreachable.
    pub fn unroot<T: Trace>(&mut self, id: Gc<T>) {
        self.roots.remove(&id.id);
    }

    // Collects unreachable objects.
    pub fn collect(&mut self) {
        println!("collecting");

        let mut queue = VecDeque::new();

        for &root in &self.roots {
            queue.push_back(root);
            // queue.push_back(self.map.get_mut(root).unwrap());
        }

        while let Some(id) = queue.pop_front() {
            let object = self.get_mut_object(id);
            // if object.reachable { continue }
            object.reachable = true;
            for id in object.value.trace() {
                let object = self.get_mut_object(id);
                if !object.reachable {
                    queue.push_back(id)
                }
            }
        }

        self.map.retain(|_, object| object.reachable);
        self.map.shrink_to_fit();
        self.map
            .values_mut()
            .for_each(|object| object.reachable = false);
        self.capacity = self.map.len() * 2 + 1;
    }
}

#[cfg(test)]
mod test {
    use super::{Gc, Heap, Id, Trace, Strategy};

    enum Tree<T: 'static> {
        Leaf(T),
        Branch(Gc<Tree<T>>, Gc<Tree<T>>),
    }

    impl<T> Trace for Tree<T> {
        fn trace(&self) -> Vec<Id> {
            match self {
                Tree::Leaf(_) => vec![],
                Tree::Branch(l, r) => vec![l.id, r.id],
            }
        }
    }

    fn mutate(gc: &mut Heap, tree: Gc<Tree<&str>>) {
        match gc.get_mut(tree) {
            Tree::Leaf(msg) => match *msg {
                "hello" => *msg = "goodbye",
                _ => (),
            },
            Tree::Branch(l, r) => {
                let (l, r) = (*l, *r);
                mutate(gc, l);
                mutate(gc, r);
            }
        }
    }

    fn print(gc: &Heap, tree: Gc<Tree<&str>>) {
        match gc.get(tree) {
            Tree::Leaf(msg) => print!("{msg}"),
            Tree::Branch(l, r) => {
                let (l, r) = (*l, *r);
                print(gc, l);
                print!(" ");
                print(gc, r);
            }
        }
    }

    fn println(gc: &mut Heap, tree: Gc<Tree<&str>>) {
        print(gc, tree);
        println!();
    }

    #[test]
    fn gc_cycle_collection_terminates() {
        let mut gc = Heap::new(Strategy::Aggressive);

        struct Cycle(Option<Gc<Cycle>>);

        impl Trace for Cycle {
            fn trace(&self) -> Vec<Id> {
                match self.0 {
                    None => vec![],
                    Some(id) => vec![id.id],
                }
            }
        }

        let mut gc = Heap::new(Strategy::Aggressive);
        let cycle = gc.alloc(Cycle(None));
        gc.get_mut(cycle).0 = Some(cycle);
        gc.collect();
    }

    #[test]
    fn gc_works() {
        let mut gc = Heap::new(Strategy::Aggressive);
        let hi = gc.alloc(Tree::Leaf("hello"));
        gc.root(hi);
        let world = gc.alloc(Tree::Leaf("world"));
        gc.root(world);
        let exclamation_mark = gc.alloc(Tree::Leaf("!"));
        gc.root(exclamation_mark);
        let greeting = gc.alloc(Tree::Branch(hi, world));
        gc.root(greeting);
        gc.unroot(hi);
        gc.unroot(world);
        let exclamation = gc.alloc(Tree::Branch(greeting, exclamation_mark));
        gc.root(exclamation);
        gc.unroot(greeting);
        gc.unroot(exclamation_mark);
        gc.collect();

        mutate(&mut gc, exclamation);

        let (left, right) = match gc.get(exclamation) {
            Tree::Branch(l, r) => (*l, *r),
            _ => unreachable!(),
        };

        println(&mut gc, exclamation);
        println(&mut gc, left);
        println(&mut gc, right);
    }
}
