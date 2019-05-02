use crate::order::{Order, Direction};
use crate::{InsertableQueue, GoodEnoughQueue};
use crate::queues::{TruncatableQueue, IterableQueue};
use std::slice;

#[derive(Clone)]
pub struct SimpleVecQueue<D>(Vec<Order<D>>);

impl<D> Default for SimpleVecQueue<D> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<D: Direction> InsertableQueue<Order<D>> for SimpleVecQueue<D> {
    fn insert_position<P>(&self, predicate: P) -> Option<usize>
        where P: FnMut(&Order<D>) -> bool
    {
        self.0.iter().position(predicate)
    }

    fn push_back(&mut self, item: Order<D>) {
        self.0.push(item)
    }

    fn insert_at(&mut self, index: usize, item: Order<D>) {
        self.0.insert(index, item)
    }
}

impl<D> TruncatableQueue for SimpleVecQueue<D> {
    fn drop_first_n(&mut self, count: usize) {
        for _ in 0..count {
            self.0.pop();
        }
    }
}

impl<D> IterableQueue<Order<D>> for SimpleVecQueue<D> {
    fn iterate<P>(&mut self, mut predicate: P) where P: FnMut(&mut Order<D>, usize) -> bool {
        for (index, order) in self.0.iter_mut().enumerate() {
            if !predicate(order, index) {
                break;
            }
        }
    }
}

impl<'a, D> IntoIterator for &'a SimpleVecQueue<D> {
    type Item = &'a Order<D>;
    type IntoIter = slice::Iter<'a, Order<D>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<D: Direction> GoodEnoughQueue<D> for SimpleVecQueue<D> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
