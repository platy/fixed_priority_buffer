use std::mem;

enum NodeOption<T> {
    Free(Option<usize>),
    Occupied {
        next: Option<usize>,
        prev: Option<usize>,
        value: T
    }
}

impl<T> NodeOption<T> {
    fn expect_free(&self) -> Option<usize> {
        match self {
            &NodeOption::Free(next) => next,
            &NodeOption::Occupied {..} => panic!["Expected Node to be free"],
        }
    }
}

enum Sentry {
    Empty,
    Filled {
        first: usize,
        last: usize
    }
}

pub struct FixedCapacityList<T> {
    heap: Vec<NodeOption<T>>,
    list: Sentry,
    free: Option<usize>,
}

impl<T> FixedCapacityList<T> {
    /// Creates a new fixed capacity list with the given capacity
    pub fn new(capacity: usize) -> FixedCapacityList<T> {
        let mut heap = Vec::<NodeOption<T>>::with_capacity(capacity);
        for i in 0..capacity-1 {
            heap.push(NodeOption::Free(Some(i+1)));
        }
        heap.push(NodeOption::Free(None));
        FixedCapacityList {
            heap: heap,
            list: Sentry::Empty,
            free: Some(0), 
        }
    }

    /// Add an element to the end of the list
    /// 
    /// # Panics
    /// if there is no remaining capacity
    pub fn enqueue(&mut self, element: T) {
        let free_index = self.free.expect("No remaining capacity");
        self.free = self.heap[free_index].expect_free();
        match self.list {
            // First item in empty list
            Sentry::Empty => {
                self.heap[free_index] = NodeOption::Occupied {
                    next: None,
                    prev: None,
                    value: element,
                };
                self.list = Sentry::Filled {
                    first: free_index,
                    last: free_index,
                }
            },
            Sentry::Filled { first, last } => {
                match self.heap[last] {
                    NodeOption::Occupied { ref mut next, .. } =>
                        *next = Some(free_index),
                    _ => panic!["Node in list was free"],
                };
                self.heap[free_index] = NodeOption::Occupied {
                    next: None,
                    prev: Some(last),
                    value: element,
                };
                self.list = Sentry::Filled {
                    first: first,
                    last: free_index,
                }
            },
        }
    }

    /// Remove and return an element from the front of the list
    pub fn dequeue(&mut self) -> Option<T> {
        match self.list {
            Sentry::Empty => None,
            Sentry::Filled { first, last } => {
                // swap the node being removed for a Free node
                let mut temp_node = NodeOption::Free(self.free);
                mem::swap(&mut self.heap[first], &mut temp_node);

                // that node is now the next free node
                self.free = Some(first);

                // we now process the occupied node and fix the list
                match temp_node {
                    // in normal case
                    NodeOption::Occupied { next: Some(next), prev: None, value } => {
                        // the next node becomes the new first node
                        self.list = match self.heap[next] {
                            NodeOption::Occupied { next: _, ref mut prev, .. } => {
                                *prev = None;
                                Sentry::Filled { first: next, last: last }            
                            },
                            _ => panic!["Free node in list"],
                        };
                        Some(value)
                    },
                    // when it was the only node
                    NodeOption::Occupied { next: None, prev: None, value } => {
                        // the list becomes empty
                        self.list = Sentry::Empty;
                        Some(value)
                    },
                    NodeOption::Occupied { .. } =>
                        panic!["removed node not at front of list"],
                    NodeOption::Free(..) => 
                        panic!["Unoccupied node in list"],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FixedCapacityList;

    #[test]
    fn list_is_has_fifo_interface() {
        let mut b = FixedCapacityList::<i32>::new(2);
        b.enqueue(1);
        b.enqueue(2);
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), None);
    }

    #[test]
    fn list_can_add_up_to_its_capacity() {
        let mut b = FixedCapacityList::<i32>::new(2);
        b.enqueue(1);
        b.enqueue(2);
    }

    #[test]
    #[should_panic]
    fn list_wont_add_past_fixed_capacity() {
        let mut b = FixedCapacityList::<i32>::new(2);
        b.enqueue(1);
        b.enqueue(2);
        b.enqueue(3);
    }

    #[test]
    fn list_reuses_freed_nodes() {
        let mut b = FixedCapacityList::<i32>::new(2);
        b.enqueue(1);
        b.enqueue(2);
        b.dequeue();
        b.dequeue();
        b.enqueue(3);
        b.enqueue(4);
        assert_eq![b.dequeue(), Some(3)];
        assert_eq![b.dequeue(), Some(4)];
    }
}

