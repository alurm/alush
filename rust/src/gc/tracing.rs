use std::ptr::null_mut;

mod gc {
    pub struct Node<T> {
        pub value: T,
        pub reachable: bool,
        next: *mut Node<T>,
    }

    pub trait Mark: Sized {
        unsafe fn mark(_: *mut Node<Self>);
    }

    #[derive(Debug)]
    pub struct Gc<T: Mark> {
        list: *mut Node<T>,
    }

    impl<T: Mark> Gc<T> {
        pub fn new() -> Self {
            Gc {
                list: std::ptr::null_mut()
            }
        }

        pub fn alloc(&mut self, init: T) -> *mut Node<T> {
            let node = Node {
                value: init,
                next: self.list,
                reachable: false,
            };

            let ptr = Box::into_raw(Box::new(node));

            self.list = ptr;

            ptr
        }

        #[allow(unsafe_op_in_unsafe_fn)]
        pub unsafe fn mark_and_sweep(&mut self, roots: &[*mut Node<T>]) {
            for &node in roots {
                Mark::mark(node);
            }
            self.sweep();
        }

        #[allow(unsafe_op_in_unsafe_fn)]
        pub unsafe fn sweep(&mut self) {
            let mut field_to_current = &raw mut (*self).list;

            while !(*field_to_current).is_null() {
                if !(**field_to_current).reachable {
                    let unreachable = *field_to_current;
                    // Update field.
                    *field_to_current = (**field_to_current).next;
                    drop(Box::from_raw(unreachable));
                } else {
                    // Go on.
                    (**field_to_current).reachable = false;
                    field_to_current = &raw mut (**field_to_current).next;
                }
            }
        }
    }
}

mod list {
    use std::ptr::null_mut;

    use super::gc;

    pub struct Node {
        pub value: i32,
        pub next: *mut super::gc::Node<Node>,
    }

    impl Node {
        pub fn new(value: i32) -> Self {
            Node {
                value: value,
                next: null_mut()
            }
        }
    }

    impl gc::Mark for Node {
        #[allow(unsafe_op_in_unsafe_fn)]
        unsafe fn mark(mut ptr: *mut gc::Node<Node>) {
            while !ptr.is_null() {
                if (*ptr).reachable {
                    return
                }
                (*ptr).reachable = true;
                ptr = (*ptr).value.next;
            }
        }
    }
}

#[test]
#[ignore]
fn gc_linked_list() {
    let mut gc = gc::Gc::<list::Node>::new();

    let mut a = gc.alloc(list::Node::new(0));
    let mut b = gc.alloc(list::Node::new(1));
    let mut c = gc.alloc(list::Node::new(2));
    let mut d = gc.alloc(list::Node::new(3));

    unsafe {
        (*a).value.next = b;
        (*c).value.next = d;
        (*d).value.next = c;

        // Will free "a" and "b".
        gc.mark_and_sweep(&[c]);
    }
}

// #[test]
// #[ignore]
// fn gc() {
//     let mut gc = gc::Gc::<i32>::new();

//     gc.alloc(0);
//     gc.alloc(1);
//     let p = gc.alloc(2);

//     unsafe {
//         (*p).reachable = true;
//     }

//     dbg!(&gc);

//     unsafe { gc.sweep(); };

//     dbg!(&gc);
// }
