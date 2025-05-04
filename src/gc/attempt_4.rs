mod linked_list {
    use std::{cell::RefCell, rc::Rc};

    struct Node<T> {
        value: T,
        next: List<T>,
    }

    type List<T> = Option<Rc<RefCell<Node<T>>>>;

    #[test]
    #[ignore]
    fn test_linked_list() {
        let a = None;
        let b = Some(Rc::new(RefCell::new(Node { value: 1, next: a })));
        let c = Some(Rc::new(RefCell::new(Node { value: 2, next: b })));
        let mut d = Some(Rc::new(RefCell::new(Node { value: 3, next: c })));

        let mut cursor = d.clone();

        while let Some(ref node) = cursor.clone() {
            let node = node.borrow();
            dbg!(node.value);
            cursor = node.next.clone();
        }

        if let Some(node) = d.clone() {
            node.borrow_mut().next = None;
        }
    }
}
