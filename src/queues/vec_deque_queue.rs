use crate::order::{Order, Direction};
use std::collections::vec_deque::VecDeque;
use crate::{InsertableQueue, GoodEnoughQueue};
use crate::queues::{IterableQueue, TruncatableQueue};

#[derive(Clone)]
pub struct VecDequeQueue<D>(VecDeque<Order<D>>);

impl<D> Default for VecDequeQueue<D> {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl<D: Direction> InsertableQueue<Order<D>> for VecDequeQueue<D> {
    fn insert_position<P>(&self, predicate: P) -> Option<usize>
        where P: FnMut(&Order<D>) -> bool
    {
        self.0.iter().position(predicate)
    }

    fn push_back(&mut self, item: Order<D>) {
        self.0.push_back(item)
    }

    fn insert_at(&mut self, index: usize, item: Order<D>) {
        self.0.insert(index, item)
    }
}

impl<D> TruncatableQueue for VecDequeQueue<D> {
    fn drop_first_n(&mut self, count: usize) {
        self.0.drain(0..count);
    }
}

impl<D> IterableQueue<Order<D>> for VecDequeQueue<D> {
    fn iterate<P>(&mut self, mut predicate: P) where P: FnMut(&mut Order<D>, usize) -> bool {
        for (index, order) in self.0.iter_mut().enumerate() {
            if !predicate(order, index) {
                break;
            }
        }
    }
}

impl<D: Direction> GoodEnoughQueue<D> for VecDequeQueue<D> {
    fn len(&self) -> usize {
        self.0.len()
    }
}