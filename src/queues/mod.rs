pub trait InsertableQueue<T> {
    fn insert_position<P>(&self, predicate: P) -> Option<usize>
        where P: FnMut(&T) -> bool;

    fn push_back(&mut self, item: T);

    fn push_front(&mut self, item: T) {
        self.insert_at(0, item)
    }

    fn insert_at(&mut self, index: usize, item: T);
}

pub trait TruncatableQueue {
    fn drop_first_n(&mut self, count: usize);
}

pub trait IterableQueue<T> {
    fn iterate<P>(&mut self, predicate: P) where P: FnMut(&mut T, usize) -> bool;
}

mod simple_vec_queue;
mod vec_deque_queue;

pub use self::simple_vec_queue::SimpleVecQueue;
pub use self::vec_deque_queue::VecDequeQueue;