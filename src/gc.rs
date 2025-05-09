/// A garbage collector.

// The idea in this iteration is to use std::any::Any.

use std::{
    any::Any, collections::{HashMap, HashSet, VecDeque}, marker::PhantomData, sync::Mutex
};

impl<T: Trace> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Trace> Copy for Gc<T> {}

static COUNTER: Mutex<usize> = Mutex::new(0);

pub struct Object {
    value: Box<dyn Trace>,
    reachable: bool,
    // Used by aggressive and checking strategies during access.
    pub alive: bool,
}

/// An opaque id for a value in the GC's heap.
/// Useful for implementing the [Trace] trait.
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Id {
    heap: usize,
    index: usize,
}

/// A trait to trace though a GC'ed value.
pub trait Trace: Any {
    fn trace(&self) -> Vec<Id>;
}

/// A handle to a value allocated in the GC's [Heap].
pub struct Gc<T: Trace> {
    pub id: Id,
    phantom_data: PhantomData<T>,
}

/// An owner of [Gc]s.
pub struct Heap {
    pub map: HashMap<Id, Object>,
    pub roots: HashMap<Id, usize>,
    counter: usize,
    capacity: usize,
    id: usize,
    strategy: Strategy,
}

pub enum Strategy {
    Disabled,
    Default,
    // Shouldn't actually delete objects at this point.
    // Instead a check for deadness is made.
    Aggressive,
    // Supposed to check for overroots and underroots.
    Checking,
}

impl Heap {
    pub fn new(strategy: Strategy) -> Self {
        Heap {
            capacity: 0,
            map: HashMap::new(),
            roots: HashMap::new(),
            counter: 0,
            strategy,
            id: {
                let mut guard = COUNTER.lock().unwrap();
                *guard = *guard + 1;
                *guard
            }
        }
    }

    pub fn rooted<T: Trace>(&mut self, init: T) -> Gc<T> {
        let object = self.alloc(init);
        self.root(object);
        object
    }

    pub fn alloc<T: Trace>(&mut self, init: T) -> Gc<T> {
        match self.strategy {
            Strategy::Aggressive | Strategy::Checking => self.collect(),
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
            alive: true,
        };

        self.map.insert(id, object);

        let object = Gc { id, phantom_data: PhantomData };

        object
    }

    fn get_object(&self, id: Id) -> &Object {
        assert!(self.id == id.heap);
        let object = self.map.get(&id).unwrap();
        if !object.alive { panic!() };
        object
    }

    fn get_mut_object(&mut self, id: Id) -> &mut Object {
        assert!(self.id == id.heap);
        let object = self.map.get_mut(&id).unwrap();
        if !object.alive { panic!() };
        object
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
        if let Some(value) = self.roots.get_mut(&id.id) {
            *value += 1;
        } else {
            self.roots.insert(id.id, 1);
        }
    }

    /// Allows a [Gc] to be collected, if discovered to be unreachable.
    pub fn unroot<T: Trace>(&mut self, id: Gc<T>) {
        if let Some(value) = self.roots.get_mut(&id.id) {
            // assert!(*value != 0);
            *value -= 1;
            if *value == 0 {
                self.roots.remove(&id.id);
            }
        } else if let Strategy::Checking = self.strategy {
            panic!()
        }
    }

    // Collects unreachable objects.
    pub fn collect(&mut self) {
        if let Strategy::Checking = self.strategy {
            println!("collecting");
        }

        let mut queue = VecDeque::new();

        for (&root, _) in &self.roots {
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

        // If we are [Aggressive] or [Checking], don't actually delete objects.
        match self.strategy {
            Strategy::Default | Strategy::Disabled => {
                self.map.retain(|_, object| object.reachable)
            }
            Strategy::Aggressive | Strategy::Checking => {
                for (_, object) in &mut self.map {
                    if !object.reachable {
                        object.alive = false;
                    }
                }
            }
        }

        self.map.shrink_to_fit();
        self.roots.shrink_to_fit();
        self.map
            .values_mut()
            .for_each(|object| object.reachable = false);
        self.capacity = self.map.len() * 2 + 1;
    }
}

impl<T: Trace> Trace for Option<T> {
    fn trace(&self) -> Vec<Id> {
        if let Some(value) = self {
            value.trace()
        } else {
            vec![]
        }
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
