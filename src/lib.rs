#![allow(unused)]

use crate::queues::{InsertableQueue, IterableQueue, TruncatableQueue};
use crate::queues::{SimpleVecQueue, VecDequeQueue};
use crate::order::{OrderSide, Order, OrderKind, IncomingOrder, Direction, Buy, Sell, TaggedOrder};
use crate::log::{ExecutionLogger, LogItem, DummyLogger};

pub mod log;
pub mod order;
pub mod queues;


pub trait OrderQueueInsert<D: Direction> {
    fn insert(&mut self, order: Order<D>);
}

pub trait OrderQueueMatch<D: Direction> {
    fn match_order(&mut self, order: &mut Order<D::Other>, kind: OrderKind, logger: &mut impl ExecutionLogger);
}

pub trait GoodEnoughQueue<D: Direction>: Default + OrderQueueInsert<D> + OrderQueueMatch<D> {
    fn len(&self) -> usize;
}

impl<D: Direction, Q: InsertableQueue<Order<D>>> OrderQueueInsert<D> for Q {
    fn insert(&mut self, order: Order<D>) {
        match D::SIDE {
            OrderSide::Buy => {
                let index = self.insert_position(|o| o.price_limit < order.price_limit);
                if let Some(index) = index {
                    self.insert_at(index, order);
                } else {
                    self.push_back(order);
                }
            }
            OrderSide::Sell => {
                let index = self.insert_position(|o| o.price_limit > order.price_limit);
                if let Some(index) = index {
                    self.insert_at(index, order);
                } else {
                    self.push_back(order);
                }
            }
        }
    }
}

impl<D: Direction, Q> OrderQueueMatch<D> for Q
where Q: IterableQueue<Order<D>> + InsertableQueue<Order<D>> + TruncatableQueue {
    fn match_order(&mut self, order: &mut Order<D::Other>, kind: OrderKind, logger: &mut impl ExecutionLogger) {
        let initial_size = order.size;
        let mut retained = Vec::new();
        let mut drop_first = 0;

        self.iterate(|passive_order, index| {
            if !passive_order.price_matches(order) {
                return false;
            }

            if passive_order.user_id == order.user_id {
                retained.push(passive_order.clone());
                return true;
            }

            let size = std::cmp::min(order.size, passive_order.size);
            order.size -= size;

            logger.log(LogItem::Fulfilled {
                size,
                price: passive_order.price_limit,
                user_id: passive_order.user_id,
            });

            if passive_order.size == size {
                drop_first = index + 1;
            } else {
                drop_first = index;
            }

            if order.size == 0 {
                passive_order.size -= size;
                return false;
            }
            true
        });

        if kind == OrderKind::FillOrKill && order.size != 0 {
            // Cancel order
            logger.cancel();
            order.size = initial_size;
            return;
        }

        if drop_first > 0 {
            self.drop_first_n(drop_first);
        }
        for order in retained.into_iter().rev() {
            self.push_front(order);
        }
    }
}


#[derive(Clone)]
pub struct OrderBook {
    bid: VecDequeQueue<Buy>,
    ask: VecDequeQueue<Sell>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bid: VecDequeQueue::default(),
            ask: VecDequeQueue::default(),
        }
    }

    pub fn bid(&self) -> &VecDequeQueue<Buy> {
        &self.bid
    }

    pub fn ask(&self) -> &VecDequeQueue<Sell> {
        &self.ask
    }

    pub fn execute_order(&mut self, order: IncomingOrder, logger: &mut impl ExecutionLogger) {
        let kind = order.kind;
        let mut order = TaggedOrder::from(order);

        match order {
            TaggedOrder::Buy(ref mut order) => self.ask.match_order(order, kind, logger),
            TaggedOrder::Sell(ref mut order) => self.bid.match_order(order, kind, logger),
        }

        let size = order.size();
        if size > 0 {
            match kind {
                OrderKind::Limit => {
                    logger.log(LogItem::Enqueued {
                        size
                    });
                    match order {
                        TaggedOrder::Buy(order) => self.bid.insert(order),
                        TaggedOrder::Sell(order) => self.ask.insert(order),
                    }
                },
                OrderKind::FillOrKill => {
                    logger.log(LogItem::Cancelled {
                        size
                    });
                },
                OrderKind::ImmediateOrCancel => {
                    logger.log(LogItem::Cancelled {
                        size
                    });
                },
            }
        }
    }

    pub fn serialize(&self) -> Vec<IncomingOrder> {
        let mut orders = Vec::new();
        for order in (&self.bid).into_iter().rev() {
            orders.push(order.to_incoming());
        }
        for order in (&self.ask).into_iter() {
            orders.push(order.to_incoming());
        }
        orders
    }

    pub fn deserialize(orders: Vec<IncomingOrder>) -> Self {
        let mut book = Self::new();
        let mut logger = DummyLogger;
        for order in orders {
            book.execute_order(order, &mut logger);
        }
        book
    }
}

#[allow(unused)]
pub fn dump20(book: &OrderBook) {
    println!("== ORDER BOOK START");
    for (index, order) in (&book.ask).into_iter().enumerate().rev() {
        if index < 25 {
            println!("{:?}", order);
        }
    }
    println!("--");
    for (index, order) in (&book.bid).into_iter().enumerate() {
        if index < 25 {
            println!("{:?}", order);
        }
    }
    println!("== ORDER BOOK END");
}

#[allow(unused)]
pub fn create_orders() -> Vec<IncomingOrder> {
    let price = 10000;
    let mut orders = Vec::new();

    let mut user_id = 10;
    for i in 0..3500 {
        user_id += 1;
        let order = IncomingOrder {
            price_limit: price + i + 1,
            size: 10,
            user_id,
            kind: OrderKind::Limit,
            side: OrderSide::Sell
        };
        orders.push(order);
        user_id += 1;
        let order = IncomingOrder {
            price_limit: price - i,
            size: 10,
            user_id,
            kind: OrderKind::Limit,
            side: OrderSide::Buy
        };
        orders.push(order);
    }
    orders
}

#[cfg(test)]
pub mod tests {
    use crate::order::*;
    use crate::log::DummyLogger;
    use crate::{OrderBook, GoodEnoughQueue};
    use super::create_orders;

    fn get_order<'a, D: 'a+Direction>(queue: impl IntoIterator<Item=&'a Order<D>>, index: usize) -> IncomingOrder {
        queue.into_iter().nth(index).expect("invalid order index").to_incoming()
    }

    fn check_order<'a, D: 'a+Direction>(queue: impl IntoIterator<Item=&'a Order<D>>, side: &str, index: usize, s: &str) {
        let order = get_order(queue, index);
        if order.to_string() != s {
            panic!("Invalid {} order at index {}: {}, should be {}", side, index, order, s);
        }
        assert_eq!(order.to_string(), s);
    }

    fn check_len<D: Direction>(queue: &impl GoodEnoughQueue<D>, side: &str, n: usize) {
        if queue.len() != n {
            panic!("Invalid {} queue length: {}, should be {}", side, queue.len(), n);
        }
    }

    pub trait OrderBookExt {
        fn check_bid(&self, index: usize, s: &str);

        fn check_bid_list(&self, list: &[&str]);

        fn check_ask(&self, index: usize, s: &str);

        fn check_ask_list(&self, list: &[&str]);

        fn check_bid_len(&self, n: usize);

        fn check_ask_len(&self, n: usize);

        fn from_orders(list: &[&str]) -> Self;
    }

    impl OrderBookExt for OrderBook {
        fn check_bid(&self, index: usize, s: &str) {
            check_order(&self.bid, "bid", index, s);
        }

        fn check_bid_list(&self, list: &[&str]) {
            check_len(&self.bid, "bid", list.len());
            for (index, s) in list.iter().enumerate() {
                check_order(&self.bid, "bid", index, s);
            }
        }

        fn check_ask(&self, index: usize, s: &str) {
            check_order(&self.ask, "ask", index, s);
        }

        fn check_ask_list(&self, list: &[&str]) {
            check_len(&self.ask, "ask", list.len());
            for (index, s) in list.iter().enumerate() {
                check_order(&self.ask, "ask", index, s);
            }
        }

        fn check_bid_len(&self, n: usize) {
            check_len(&self.bid, "bid", n);
        }

        fn check_ask_len(&self, n: usize) {
            check_len(&self.ask, "ask", n);
        }

        fn from_orders(list: &[&str]) -> Self {
            let mut logger = DummyLogger;

            let mut book = OrderBook::new();
            for s in list {
                book.execute_order(s.parse().unwrap(), &mut logger);
            }
            book
        }
    }

    #[test]
    fn new_book_is_empty() {
        let book = OrderBook::new();
        book.check_bid_len(0);
        book.check_ask_len(0);
    }

    #[test]
    fn book_insert_correct_queue() {
        let mut logger = DummyLogger;

        let mut book = OrderBook::new();
        let order = "Lim B $100 #200 u42".parse().unwrap();
        book.execute_order(order, &mut logger);
        book.check_bid_len(1);
        book.check_ask_len(0);

        let mut book = OrderBook::new();
        let order = "Lim S $100 #200 u42".parse().unwrap();
        book.execute_order(order, &mut logger);
        book.check_bid_len(0);
        book.check_ask_len(1);
    }

    #[test]
    fn book_insert_correct_ordering_by_price() {
        let orders = [
            "Lim B $110 #100 u42",
            "Lim B $130 #100 u42",
            "Lim B $120 #100 u42",
            "Lim B $100 #100 u42",
        ];
        let book = OrderBook::from_orders(&orders);
        book.check_bid_list(&[
            orders[1],
            orders[2],
            orders[0],
            orders[3],
        ]);

        let orders = [
            "Lim S $110 #100 u42",
            "Lim S $130 #100 u42",
            "Lim S $120 #100 u42",
            "Lim S $100 #100 u42",
        ];
        let book = OrderBook::from_orders(&orders);
        book.check_ask_list(&[
            orders[3],
            orders[0],
            orders[2],
            orders[1],
        ]);
    }

    #[test]
    fn book_insert_correct_ordering_by_arrival() {
        let orders = [
            "Lim B $100 #100 u41",
            "Lim B $101 #100 u42",
            "Lim B $102 #100 u43",
            "Lim B $101 #100 u44",
            "Lim B $101 #100 u45",
        ];
        let book = OrderBook::from_orders(&orders);
        book.check_bid_list(&[
            orders[2],
            orders[1],
            orders[3],
            orders[4],
            orders[0],
        ]);

        let orders = [
            "Lim S $100 #100 u41",
            "Lim S $101 #100 u42",
            "Lim S $102 #100 u43",
            "Lim S $101 #100 u44",
            "Lim S $101 #100 u45",
        ];
        let book = OrderBook::from_orders(&orders);
        book.check_ask_list(&[
            orders[0],
            orders[1],
            orders[3],
            orders[4],
            orders[2],
        ]);
    }

    #[test]
    fn matching_with_20_orders() {
        let orders = create_orders();
        let mut book = OrderBook::deserialize(orders);
        let mut logger = DummyLogger;
        book.check_bid_len(3500);
        book.check_ask_len(3500);

        let order = IncomingOrder {
            price_limit: 10020,
            size: 200,
            user_id: 0,
            kind: OrderKind::Limit,
            side: OrderSide::Buy
        };
        book.execute_order(order, &mut logger);
        book.check_bid_len(3500);
        book.check_ask_len(3500-20);
    }
}
