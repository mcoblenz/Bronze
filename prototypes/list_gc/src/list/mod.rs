use bronze_gc::*;

// Implements a doubly-linked list.
// Lists live in the GC heap but are intended to be referenced from outside.

pub struct List<T>
where T: 'static + GcTrace
{
    head: Option<GcNullableRef<Node<T>>>,
}

impl<T: GcTrace> Finalize for List<T> {}
unsafe impl<T: GcTrace> GcTrace for List<T> {
    custom_trace!(this, {
        match this.head {
            None => (),
            Some(n) => mark(&n),
        }
    });
}

pub struct Node<T> 
where T: 'static + GcTrace {
    value: T,

    prev: Option<GcNullableRef<Node<T>>>,
    next: Option<GcNullableRef<Node<T>>>,
}

impl<T: GcTrace> Finalize for Node<T> {}
unsafe impl<T: GcTrace> GcTrace for Node<T> {
    custom_trace!(this, {
        mark(&this.value);

        match this.next {
            None => (),
            Some(n) => mark(&n),
        }
    });
}

impl<T: GcTrace> Node<T> {
    pub fn len(&self) -> usize {
        match self.next {
            None => 1,
            Some(n) => {
                1 + n.as_ref().len()
            }
        }
    }
}

impl<T: GcTrace> List<T> {
    pub fn new() -> GcNullableRef<Self> {
        Gc::new_nullable(List {head: None})
    }

    pub fn len(self_ref: GcNullableRef<Self>) -> usize {
        match &self_ref.as_ref().head {
            None => 0,
            Some(n) => n.as_ref().len()
        }
    }

    pub fn push(self_ref: &mut GcNullableRef<Self>, value: T) {
        let list = self_ref.as_mut();
         match &mut list.head {
             None => {
                 let new_head = Node {
                    value,
                    prev: None,
                    next: None,
                 };
                 list.head = Some(Gc::new_nullable(new_head));
             },
             Some(orig_head) => {
                 let new_head = Node {
                     value,
                     prev: None,
                     next: Some(*orig_head),
                 };

                 let new_head_gc = Gc::new_nullable(new_head);

                 assert!(orig_head.as_ref().prev.is_none());
                 orig_head.as_mut().prev = Some(new_head_gc);
                 list.head = Some(new_head_gc);
             }
         }
    }

    pub fn pop(self_ref: &mut GcNullableRef<Self>) -> Option<T> {
        let list = self_ref.as_mut();

        // Will need to update head_ref with the new head.
        let head_opt_ref = &mut list.head;
        
        head_opt_ref.map(|mut h| {
            let head = h.remove();

            match head {
                None => panic!("Bug in List::pop. Tried to eliminate a node twice."),
                Some(Node {value, prev: _, next}) => {
                    list.head = next;
                    value
                }
            }
        })
    }    

    pub fn assert_consistency(self_ref: GcNullableRef<Self>) {
        let list = self_ref.as_ref();
        match &list.head {
            None => {},
            Some(h) => {
                match h.as_ref().next {
                    None => (),
                    Some(next) => {
                        match next.as_ref().prev {
                            None => assert!(false),
                            Some(p) => {
                                assert!(p == *h);
                            }
                        }
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::list::*;

    #[test]
    fn empty_list() {
        let empty_list = List::<u32>::new();
        assert_eq!(List::len(empty_list), 0);
        List::assert_consistency(empty_list);
        
    }

    #[test]
    fn list_push() {
        let mut l = List::<u32>::new();
        List::push(&mut l, 1);
        List::push(&mut l, 2);
        assert_eq!(List::len(l), 2);
        List::assert_consistency(l);
    }

    #[test]
    fn list_pop() {
        let mut l = List::<u32>::new();
        List::push(&mut l, 1);
        List::push(&mut l, 2);
        assert_eq!(List::len(l), 2);
        List::assert_consistency(l);
        let two = List::pop(&mut l);
        assert_eq!(two, Some(2));
        let one = List::pop(&mut l);
        assert_eq!(one, Some(1));
        let empty1 = List::pop(&mut l);
        assert_eq!(empty1, None);
        let empty2 = List::pop(&mut l);
        assert_eq!(empty2, None);
    }

    #[test]
    fn list_of_lists() {
        // should be "new List<# List<u32>>"
        let mut l = List::<GcNullableRef<List<u32>>>::new();

        // a Copy reference to a list
        // should be "new List<u32>" with syntactic sugar
        let mut empty_list = List::<u32>::new();

        // should be l.push(empty_list)
        List::push(&mut l, empty_list);
        List::push(&mut l, empty_list);

        List::push(&mut empty_list, 42);

        let fortytwo_list = List::pop(&mut l);
        match fortytwo_list {
            None => assert!(false, "fortytwo_list should have been popped off"),
            Some(mut list_contents) => {
                assert_eq!(List::len(list_contents), 1);
                let fortytwo = List::pop(&mut list_contents);
                assert_eq!(fortytwo, Some(42));
                List::push(&mut list_contents, 42); // put 42 back for the next test
            }
        }

        // There should be TWO references to the SAME list. Check the second one, too.
        let fortytwo_list2 = List::pop(&mut l);
        match fortytwo_list2 {
            None => assert!(false, "fortytwo_list should have been popped off"),
            Some(mut list_contents) => {
                assert_eq!(List::len(list_contents), 1);
                let fortytwo = List::pop(&mut list_contents);
                assert_eq!(fortytwo, Some(42));
            }
        }


    }
}