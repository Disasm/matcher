use crate::order::{Order, Direction};
use std::slice;
use crate::queues::Queue;

#[derive(Clone)]
pub struct SimpleVecQueue<D>(Vec<Order<D>>);

impl<D: Direction> Queue<Order<D>> for SimpleVecQueue<D> {
    fn new() -> Self {
        Self(Vec::new())
    }

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

    fn drop_first_n(&mut self, count: usize) {
        for _ in 0..count {
            self.0.pop();
        }
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


impl<'a, D> IntoIterator for &'a SimpleVecQueue<D> {
    type Item = &'a Order<D>;
    type IntoIter = slice::Iter<'a, Order<D>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
