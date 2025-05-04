// This is the fifth attempt.

use std::collections::{HashMap, HashSet, VecDeque};

/// The key for the hashmap of a GC.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Key(usize);

/// The ID for a GC object the client receives.
#[derive(Debug)]
pub struct Ptr<T> {
    // Used to assert that an ID is related to the GC.
    gc: *const Gc<T>,
    key: Key,
}

impl<T> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for Ptr<T> {}

impl<T> std::hash::Hash for Ptr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.gc.hash(state);
        self.key.hash(state);
    }
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Ptr<T> {}

/// The GC object.
struct Object<T> {
    value: T,
    reachable: bool,
}

impl<T> Object<T> {
    fn new(init: T) -> Self {
        Object {
            value: init,
            reachable: false,
        }
    }
}

pub struct Gc<T> {
    // Used to create increasing IDs for the map.
    counter: Key,
    map: HashMap<Key, Object<T>>,
    // Could be Key here. Doing what's simpler instead.
    roots: HashSet<Ptr<T>>,
    trace: fn(&T) -> Vec<Ptr<T>>,
    // How many live objects should be allocated to trigger collection.
    collect_at: usize,

    // Whether to collect, ever. Useful if tracing is incorrect.
    should_collect: bool,
}

#[derive(Debug)]
enum Tree {
    Leaf(i32),
    Branch(Ptr<Tree>, Ptr<Tree>),
}

// pub struct RootGuard<'a, T> {
//     ptr: Ptr<T>,
//     gc: &'a mut Gc<T>,
// }

// impl<'a, T> Drop for RootGuard<'a, T> {
//     fn drop(&mut self) {
//         self.gc.unroot(self.ptr);
//     }
// }

// impl<'a, T> Deref for RootGuard<'a, T> {
//     fn deref(&self) -> &Self::Target {
//         &self.ptr
//     }
//     type Target = Ptr<T>;
// }
// impl<'a, T> std::ops::DerefMut for RootGuard<'a, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.ptr   
//     }
// }

impl<T> Gc<T> {
    pub fn new(trace: fn(&T) -> Vec<Ptr<T>>) -> Self {
        Gc {
            counter: Key(0),
            map: HashMap::new(),
            roots: HashSet::new(),
            trace: trace,
            collect_at: 1,
            should_collect: true,
        }
    }

    pub fn new_noncollecting(trace: fn(&T) -> Vec<Ptr<T>>) -> Self {
        let mut gc = Self::new(trace);
        gc.should_collect = false;
        gc
    }

    pub fn alloc(&mut self, init: T) -> Ptr<T> {
        if self.map.len() == self.collect_at && self.should_collect {
            self.collect();
        }
        let key = self.counter;
        self.counter = Key(self.counter.0.checked_add(1).unwrap());
        self.map.insert(key, Object::new(init));
        Ptr {
            gc: self,
            key: key,
        }
    }

    // pub fn guarded_alloc(&mut self, init: T) -> RootGuard<'_, T> {
    //     RootGuard { ptr: self.alloc(init), gc: self }
    // }

    fn get_object(&self, id: Ptr<T>) -> &Object<T> {
        // Assert that the ID is bound to the GC.
        assert!(id.gc == self);
        &self.map[&id.key]
    }

    fn get_mut_object(&mut self, id: Ptr<T>) -> &mut Object<T> {
        // Assert that the ID is bound to the GC.
        assert!(id.gc == self);
        self.map.get_mut(&id.key).unwrap()
    }

    pub fn get(&self, id: Ptr<T>) -> &T {
        &self.get_object(id).value
    }

    pub fn get_mut(&mut self, id: Ptr<T>) -> &mut T {
        &mut self.get_mut_object(id).value
    }

    pub fn root(&mut self, id: Ptr<T>) {
        self.roots.insert(id);
    }

    pub fn unroot(&mut self, id: Ptr<T>) {
        self.roots.remove(&id);
    }

    fn mark(&mut self, id: Ptr<T>, reachable: bool) {
        self.get_mut_object(id).reachable = reachable;
    }

    /// 1. Create a queue of ids.
    /// 2. Put all root ids in the queue.
    /// 3. While queue is not empty:
    ///    1. Pop an element from the queue, mark it as reachable.
    ///    2. Add directly reachable unmarked objects to the queue.
    /// 4. Delete all unreachable objects.
    /// 5. Mark all objects as unreachable.
    /// Returns the amount of live objects left.
    pub fn collect(&mut self) {
        let mut queue = VecDeque::<Ptr<T>>::new();

        for &root in &self.roots {
            queue.push_back(root);
        }

        while !queue.is_empty() {
            let id = queue.pop_front().unwrap();
            let trace = self.trace; 
            let object = self.get_mut_object(id);
            object.reachable = true;
            for id in trace(&object.value) {
                let object = self.get_mut_object(id);
                if !object.reachable {
                    queue.push_back(id);
                }
            }
        }

        self.map.retain(|_, v| v.reachable);

        for (_, object) in &mut self.map {
            object.reachable = false;
        }

        let left = self.map.len();
        self.collect_at = left * 2 + 1;
        self.map.shrink_to_fit();
    }
}

impl Tree {
    fn trace(&self) -> Vec<Ptr<Self>> {
        match self {
            Self::Leaf(_) => Vec::new(),
            Self::Branch(l, r) => Vec::from([*l, *r]),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::Gc;

    fn gc() {
        let gc = Gc::new(|_: &i32| Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}

#[test]
#[ignore]
fn test_gc() {
    let mut gc = Gc::new(Tree::trace);

    let a = gc.alloc(Tree::Leaf(42));

    gc.collect();

    assert!(gc.map.len() == 0);

    // let a = gc.alloc("hello".to_string());
    // let b = gc.alloc("world".to_string());

    // let a = gc.alloc(Tree::Leaf(1));
    // let b = gc.alloc(Tree::Branch(a, a));
    // let c = gc.alloc(Tree::Branch(b, b));

    // {
    //     // let tree = gc.deref_mut(c);
    //     // *tree = Tree::Branch(c, c);

    //     let id = if let Tree::Branch(l, _ ) = gc.get_mut(c) {
    //         *l
    //     } else {
    //         unreachable!()
    //     };

    //     let id = if let Tree::Branch(l, _) = gc.get_mut(id) {
    //         *l
    //     } else {
    //         unreachable!()
    //     };

    //     *gc.get_mut(id) = Tree::Leaf(2);
    // }

    // dbg!(gc.get(a));

    // // gc.mark_roots();
    // // gc.root(a);
    // // gc.root(c);

    // // dbg!(b.index);

    // // gc.collect();

    // // gc.deref(c);
}
