#![allow(unused)]

use std::cmp;
use crate::queues::{InsertableQueue, IterableQueue, TruncatableQueue};
use crate::queues::{SimpleVecQueue, VecDequeQueue};
use crate::order::{OrderSide, Order, OrderKind, IncomingOrder, Direction, Buy, Sell};

pub mod order;
pub mod queues;


pub trait OrderQueueInsert<D: Direction> {
    fn insert(&mut self, order: Order<D>);
}

pub trait OrderQueueMatch<D: Direction> {
    fn match_order(&mut self, order: &mut Order<D::Other>);
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
    fn match_order(&mut self, order: &mut Order<<D as Direction>::Other>) {
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

            let amount = cmp::min(order.amount, passive_order.amount);
            order.amount -= amount;
            if passive_order.amount == amount {
                drop_first = index + 1;
            } else {
                drop_first = index;
            }

            if order.amount == 0 {
                passive_order.amount -= amount;
                return false;
            }
            true
        });
        if drop_first > 0 {
            self.drop_first_n(drop_first);
        }
        for order in retained.into_iter().rev() {
            self.push_front(order);
        }
    }
}


pub enum ExecutionResult {
    Enqueued,
    Fulfilled,
    Cancelled,
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

    fn execute_sell(&mut self, mut order: Order<Sell>) {
        self.bid.match_order(&mut order);
        if order.amount > 0 {
            self.ask.insert(order);
        }
    }

    fn execute_buy(&mut self, mut order: Order<Buy>) {
        self.ask.match_order(&mut order);
        if order.amount > 0 {
            self.bid.insert(order);
        }
    }

    pub fn execute_order(&mut self, order: IncomingOrder) {
        match order.side {
            OrderSide::Buy => self.execute_buy(order.into()),
            OrderSide::Sell => self.execute_sell(order.into()),
        }
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
            amount: 10,
            user_id,
            kind: OrderKind::Limit,
            side: OrderSide::Sell
        };
        orders.push(order);
        user_id += 1;
        let order = IncomingOrder {
            price_limit: price - i,
            amount: 10,
            user_id,
            kind: OrderKind::Limit,
            side: OrderSide::Buy
        };
        orders.push(order);
    }
    orders
}

#[test]
fn matching_with_20_orders() {
    let orders = create_orders();
    let mut book = OrderBook::new();
    for order in orders {
        book.execute_order(order);
    }
    assert_eq!(book.bid.len(), 3500);
    assert_eq!(book.ask.len(), 3500);

    let order = IncomingOrder {
        price_limit: 10020,
        amount: 200,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };
    book.execute_order(order);
    assert_eq!(book.bid.len(), 3500);
    assert_eq!(book.ask.len(), 3500-20);
}
