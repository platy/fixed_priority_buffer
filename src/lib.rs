use std::ptr;
use std::mem;

pub struct FixedSizePriorityBuffer<T> {
    capacity: usize,
    size: usize,
    // Series data
    first: Link<T>,
    last: Rawlink<Node<T>>,
    // Preallocated nodes
    first_empty: Link<T>,
    last_empty: Rawlink<Node<T>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Rawlink<T> {
    p: *mut T,
}

struct Node<T> {
    next: Link<T>,
    prev: Rawlink<Node<T>>,
    value: Option<T>,
}

impl<T> Rawlink<T> {
    fn none() -> Rawlink<T> {
        Rawlink{p: ptr::null_mut()}
    }

    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink{p: n}
    }

    unsafe fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        if self.p.is_null() {
            None
        } else {
            Some(&mut *self.p)
        }
    }

    fn is_none(&self) -> bool {
        self.p.is_null()
    }
}

impl<T> Node<T> {
    fn new(v: T) -> Node<T> {
        Node{value: Some(v), next: None, prev: Rawlink::none()}
    }

    fn empty() -> Node<T> {
        Node{value: None, next: None, prev: Rawlink::none()}
    }

    fn set_next(&mut self, mut next: Box<Node<T>>) {
        debug_assert!(self.next.is_none());
        next.prev = Rawlink::some(self);
        self.next = Some(next);
    }
}

impl<'a, T> From<&'a mut Link<T>> for Rawlink<Node<T>> {
    fn from(node: &'a mut Link<T>) -> Self {
        match node.as_mut() {
            None => Rawlink::none(),
            Some(ptr) => Rawlink::some(ptr),
        }
    }
}

fn link_no_prev<T>(mut next: Box<Node<T>>) -> Link<T> {
    next.prev = Rawlink::none();
    Some(next)
}


fn insert_slice<T>(range_start: &mut Link<T>, range_end: &mut Rawlink<Node<T>>,
                   target_start: &mut Link<T>, target_end: &mut Rawlink<Node<T>>) {
    if range_start.is_none() {
        panic!("range_start must not be none");
    }
    if range_end.is_none() {
        panic!("range_end must not be none");
    }
    

    unsafe {
        // dereference the end node first as the pointer will be changed when rotating the backwards links
        let before_end_node: *mut Node<T> = 
            range_end.resolve_mut().expect("source list to not be emptyi: end is none");
        // rotate backawrds links first as we have captured the pointer to the end node
        {
            let start_node: &mut Box<Node<T>> = &mut 
                range_start.as_mut().expect("source list to not be empty: start is none");
            let mut temp: Rawlink<Node<T>> = mem::uninitialized();
            ptr::copy_nonoverlapping(&start_node.prev, &mut temp, 1);
            ptr::copy_nonoverlapping(&*target_end, &mut start_node.prev, 1);
            ptr::copy_nonoverlapping(&*range_end, target_end, 1);
            ptr::copy_nonoverlapping(&temp, range_end, 1);

            mem::forget(temp);
        }

        // rotate forwards links
        {
            let mut temp: Link<T> = mem::uninitialized();
            ptr::copy_nonoverlapping(&*range_start, &mut temp, 1);
            ptr::copy_nonoverlapping(&(*before_end_node).next, range_start, 1); 
            ptr::copy_nonoverlapping(&*target_start, &mut (*before_end_node).next, 1);
            ptr::copy_nonoverlapping(&temp, target_start, 1); 

            mem::forget(temp);
        }
    }
}

#[cfg(test)]
mod swap_tests {
    use super::Node;
    use super::link_no_prev;
    use super::Rawlink;
    use super::insert_slice;

    #[test]
    fn swaps_single_item_list_with_empty() {
        let mut single_node = Box::new(Node::new("Single node"));
        let mut single_end = Rawlink::some(single_node.as_mut());
        let mut single_start = link_no_prev(single_node);
        let mut empty_start = None;
        let mut empty_end = Rawlink::none();

        insert_slice(&mut single_start, &mut single_end, &mut empty_start, &mut empty_end);

        assert!(single_start.is_none());
        assert!(single_end.is_none());
        let empty_start = empty_start.expect("the node should have been moved");
        assert_eq!(empty_start.value.expect("some value on node"), "Single node");
    }
}


impl<T> FixedSizePriorityBuffer<T> {
    pub fn new(capacity: usize) -> FixedSizePriorityBuffer<T> {
        let mut first_empty = Box::new(Node::<T>::empty());
        let last_empty = Rawlink::some(first_empty.as_mut());
        for _ in 1..capacity {
            let mut next_empty = Box::new(Node::<T>::empty());
            next_empty.set_next(first_empty);
            first_empty = next_empty;
        }
        FixedSizePriorityBuffer{
            capacity: capacity,
            size: 0,
            first: None,
            last: Rawlink::none(),
            first_empty: link_no_prev(first_empty),
            last_empty: last_empty,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn enqueue(&mut self, element: T) {
        if self.size >= self.capacity {
            panic!("No remaining capacity to enqueue");
        }
        let new_node = Box::new(Node::new(element));
        match unsafe { self.last.resolve_mut() } {
            None => {
                self.first = Some(new_node);
                self.last = Rawlink::from(&mut self.first);
            },
            Some(node) => {
                node.set_next(new_node);
                self.last = Rawlink::from(&mut node.next);
            },
        }
        self.size += 1;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.first.take().map(|mut first_node| {
            self.size -= 1;
            match first_node.next.take() {
                Some(node) => self.first = link_no_prev(node),
                None => self.last = Rawlink::none(),
            }
            first_node.value.expect("Value should always be Some when being dequeued")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FixedSizePriorityBuffer;

    #[test]
    fn queue_is_fifo() {
        let mut b = FixedSizePriorityBuffer::<i32>::new(3);
        assert_eq!(b.capacity(), 3);
        assert_eq!(b.size(), 0);
        b.enqueue(1);
        b.enqueue(2);
        b.enqueue(3);
        assert_eq!(b.size(), 3);
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), Some(3));
        assert_eq!(b.size(), 0);
    }

    #[test]
    #[should_panic]
    fn queue_is_fixed_capacity() {
        let mut b = FixedSizePriorityBuffer::<i32>::new(2);
        b.enqueue(1);
        b.enqueue(2);
        b.enqueue(3);
    }
}

