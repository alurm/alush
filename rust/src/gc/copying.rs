// #[derive(Clone)]
// struct ListNode<T> {
//     next: Option<usize>,
//     value: T,
// }

// impl<T: Clone> ListNode<T> {
//     fn mark(gc: &mut Gc<ListNode<T>>, i: usize) {
//         // match &self.current[i] {
//         //     GcNode::Copied(_) => {},
//         //     GcNode::Present(p) => {
//         //         let moved_to = self.next.len();
//         //         self.next.push(self.current[i].clone());
//         //         self.current[i] = GcNode::Copied(moved_to);
//         //     }
//         // }
//         match &gc.current[i] {
//             GcNode::Copied(_) => {}
//             GcNode::Present(node) => {
//                 let copied_to = gc.next.len();
//                 gc.next.push(GcNode::Present(node.clone()));
//                 gc.current[i] = GcNode::Copied(copied_to);
//                 if let GcNode::Present(node) = &gc.next[copied_to] {
//                     if let Some(i) = node.next {
//                         Self::mark(gc, i);
//                     }
//                 } else {
//                     unreachable!();
//                 }
//             }
//         }
//     }
// }

// #[derive(Clone)]
// enum GcNode<T> {
//     Copied(usize),
//     Present(T),
// }

// struct Gc<T: Clone> {
//     current: Vec<GcNode<T>>,
//     next: Vec<GcNode<T>>,
// }

// impl<T: Clone> Gc<T> {
//     fn new() -> Self {
//         Gc {
//             current: Vec::new(),
//             next: Vec::new(),
//         }
//     }

//     fn alloc(&mut self, init: T) -> usize {
//         let i = self.current.len();
//         self.current.push(GcNode::Present(init));
//         i
//     }

//     fn copy(&mut self, i: usize) {
//         match &self.current[i] {
//             GcNode::Copied(_) => {}
//             GcNode::Present(p) => {
//                 let moved_to = self.next.len();
//                 self.next.push(self.current[i].clone());
//                 self.current[i] = GcNode::Copied(moved_to);
//             }
//         }
//     }

//     fn borrow(&self, i: usize) -> &T {
//         match self.current[i] {
//             GcNode::Present(ref p) => p,
//             // GcNode::Moved(i) => panic!("")
//             //     self.borrow(i)
//             // }
//             _ => todo!(),
//         }
//     }

//     fn update_roots(&self, roots: &mut [usize]) {
//         for root in roots {
//             match &self.current[root] {
//                 GcNode::Copied(new) => *root = new,
//                 _ => unreachable!(),
//             }
//         }
//     }

//     fn collect(self) -> Self {
//         Self {
//             current: self.next,
//             next: Vec::new(),
//         }
//     }
// }

// #[test]
// #[ignore]
// fn test_list() {
//     let mut gc = Gc::<ListNode<i32>>::new();

//     let mut roots = Vec::new();

//     let d = gc.alloc(ListNode {
//         next: None,
//         value: 0,
//     });
//     let c = gc.alloc(ListNode {
//         next: None,
//         value: 0,
//     });
//     let b = gc.alloc(ListNode {
//         next: Some(c),
//         value: 1,
//     });
//     let a = gc.alloc(ListNode {
//         next: Some(b),
//         value: 2,
//     });

//     roots.push(a);

//     dbg!(gc.borrow(a).value);


//     ListNode::mark(&mut gc, a);
//     gc.copy(a);
//     gc.update_roots(roots);
//     gc = gc.collect();
//     dbg!(roots);
// }

// #[test]
// #[ignore]
// fn test_rc() {
//     use std::rc::{self, Rc, Weak};

//     let vec = std::rc::Rc::new(Vec::<i32>::new());

//     let _ = Rc::downgrade(&vec.clone());

//     let s = std::cell::RefCell::new(&vec);

//     let r = &s;
//     // let r2 = &s;

//     // r.borrow_mut().push(1);
// }
