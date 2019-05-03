use crate::order::{Order, Direction};
use crate::queues::{InsertableQueue, TruncatableQueue, IterableQueue};
use crate::GoodEnoughQueue;
use std::{slice, iter};

#[derive(Clone)]
pub struct ReversedVec<D>(Vec<Order<D>>);

impl<D> Default for ReversedVec<D> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<D: Direction> InsertableQueue<Order<D>> for ReversedVec<D> {
    fn insert_position<P>(&self, predicate: P) -> Option<usize>
        where P: FnMut(&Order<D>) -> bool
    {
        self.0.iter().rev().position(predicate)
    }

    fn push_back(&mut self, item: Order<D>) {
        self.0.insert(0, item)
    }

    fn insert_at(&mut self, index: usize, item: Order<D>) {
        if index == 0 {
            self.0.push(item)
        } else {
            self.0.insert(self.0.len() - index, item);
        }
    }
}

impl<D> TruncatableQueue for ReversedVec<D> {
    fn drop_first_n(&mut self, count: usize) {
        self.0.truncate(self.0.len() - count)
    }
}

impl<D> IterableQueue<Order<D>> for ReversedVec<D> {
    fn iterate<P>(&mut self, mut predicate: P) where P: FnMut(&mut Order<D>, usize) -> bool {
        for (index, order) in self.0.iter_mut().rev().enumerate() {
            if !predicate(order, index) {
                break;
            }
        }
    }
}

impl<'a, D> IntoIterator for &'a ReversedVec<D> {
    type Item = &'a Order<D>;
    type IntoIter = iter::Rev<slice::Iter<'a, Order<D>>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter().rev()
    }
}

impl<D: Direction> GoodEnoughQueue<D> for ReversedVec<D> {
    fn len(&self) -> usize {
        self.0.len()
    }
}