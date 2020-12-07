use bronze::*;


pub struct List<T>
where T: 'static
{
    head: Option<GcRef<Node<T>>>,
}

pub struct Node<T> 
where T: 'static {
    value: T,

    prev: Option<GcRef<Node<T>>>,
    next: Option<GcRef<Node<T>>>,
}

impl<T> GcTrace for Node<T> {

}

impl<T> Node<T> {
    pub fn len(&self) -> usize {
        match self.next {
            None => 1,
            Some(n) => {
                1 + n.as_ref().len()
            }
        }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {head: None}
    }

    pub fn len(&self) -> usize {
        match &self.head {
            None => 0,
            Some(n) => n.as_ref().len()
        }
    }

    pub fn push(&mut self, value: T) {
         match &mut self.head {
             None => {
                 let new_head = Node {
                    value,
                    prev: None,
                    next: None,
                 };
                 self.head = Some(Gc::new(new_head));
             },
             Some(orig_head) => {
                 let new_head = Node {
                     value,
                     prev: None,
                     next: Some(*orig_head),
                 };

                 let new_head_gc = Gc::new(new_head);

                 assert!(orig_head.as_ref().prev.is_none());
                 orig_head.as_mut_ref().prev = Some(new_head_gc);
                 self.head = Some(new_head_gc);
             }
         }
    }

    pub fn assert_consistency(&self) {
        match &self.head {
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
        assert_eq!(empty_list.len(), 0);
        empty_list.assert_consistency();
    }

    #[test]
    fn list_push() {
        let mut l = List::<u32>::new();
        l.push(1);
        l.push(2);
        assert_eq!(l.len(), 2);
        l.assert_consistency();
    }
}