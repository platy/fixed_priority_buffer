extern crate core;
use core::ptr;

pub struct FixedSizePriorityBuffer<T> {
    capacity: usize,
    size: usize,
    // Series data
    first: Link<T>,
    last: Rawlink<Node<T>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Rawlink<T> {
    p: *mut T,
}

struct Node<T> {
    next: Link<T>,
    prev: Rawlink<Node<T>>,
    value: T,
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
}

impl<T> Node<T> {
    fn new(v: T) -> Node<T> {
        Node{value: v, next: None, prev: Rawlink::none()}
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

impl<T> FixedSizePriorityBuffer<T> {
    #[inline]
    pub fn new(capacity: usize) -> FixedSizePriorityBuffer<T> {
        FixedSizePriorityBuffer{
            capacity: capacity,
            size: 0,
            first: None,
            last: Rawlink::none(),
        }
    }

    pub fn enqueue(&mut self, element: T) {
        let mut new_node = Box::new(Node::new(element));
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
            first_node.value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FixedSizePriorityBuffer;

    #[test]
    fn queue_is_fifo() {
        let mut b = FixedSizePriorityBuffer::<i32>::new(10);
        assert_eq!(b.capacity, 10);
        assert_eq!(b.size, 0);
        b.enqueue(1);
        b.enqueue(2);
        b.enqueue(3);
        assert_eq!(b.size, 3);
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), Some(3));
        assert_eq!(b.size, 0);
    }
}
