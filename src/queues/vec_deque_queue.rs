use crate::order::{Order, Direction};
use std::collections::vec_deque::VecDeque;
use std::collections::vec_deque;
use crate::queues::Queue;

#[derive(Clone)]
pub struct VecDequeQueue<D>(VecDeque<Order<D>>);

impl<D: Direction> Queue<Order<D>> for VecDequeQueue<D> {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn insert_position<P>(&self, predicate: P) -> Option<usize>
        where P: FnMut(&Order<D>) -> bool
    {
        self.0.iter().position(predicate)
    }

    fn push_back(&mut self, item: Order<D>) {
        self.0.push_back(item)
    }

    fn insert_at(&mut self, index: usize, item: Order<D>) {
        if index == 0 {
            self.0.push_front(item)
        } else {
            self.0.insert(index, item)
        }
    }

    fn drop_first_n(&mut self, count: usize) {
        self.0.drain(0..count);
    }

    fn iterate<P>(&mut self, mut predicate: P) where P: FnMut(&mut Order<D>, usize) -> bool {
        for (index, order) in self.0.iter_mut().enumerate() {
            if !predicate(order, index) {
                break;
            }
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}


impl<'a, D> IntoIterator for &'a VecDequeQueue<D> {
    type Item = &'a Order<D>;
    type IntoIter = vec_deque::Iter<'a, Order<D>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
