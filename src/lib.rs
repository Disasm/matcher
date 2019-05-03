#![allow(unused)]

use crate::queues::{InsertableQueue, IterableQueue, TruncatableQueue};
use crate::queues::{SimpleVecQueue, VecDequeQueue};
use crate::order::{OrderSide, Order, OrderKind, IncomingOrder, Direction, Buy, Sell};
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

    fn execute_sell(&mut self, mut order: Order<Sell>, kind: OrderKind, logger: &mut impl ExecutionLogger) {
        self.bid.match_order(&mut order, kind, logger);

        if order.size > 0 {
            match kind {
                OrderKind::Limit => {
                    logger.log(LogItem::Enqueued {
                        size: order.size
                    });
                    self.ask.insert(order);
                },
                OrderKind::FillOrKill => {
                    logger.log(LogItem::Cancelled {
                        size: order.size
                    });
                },
                OrderKind::ImmediateOrCancel => {
                    logger.log(LogItem::Cancelled {
                        size: order.size
                    });
                },
            }
        }
    }

    fn execute_buy(&mut self, mut order: Order<Buy>, kind: OrderKind, logger: &mut impl ExecutionLogger) {
        self.ask.match_order(&mut order, kind, logger);

        if order.size > 0 {
            match kind {
                OrderKind::Limit => {
                    logger.log(LogItem::Enqueued {
                        size: order.size
                    });
                    self.bid.insert(order);
                },
                OrderKind::FillOrKill => {
                    logger.log(LogItem::Cancelled {
                        size: order.size
                    });
                },
                OrderKind::ImmediateOrCancel => {
                    logger.log(LogItem::Cancelled {
                        size: order.size
                    });
                },
            }
        }
    }

    pub fn execute_order(&mut self, order: IncomingOrder, logger: &mut impl ExecutionLogger) {
        let kind = order.kind;
        match order.side {
            OrderSide::Buy => self.execute_buy(order.into(), kind, logger),
            OrderSide::Sell => self.execute_sell(order.into(), kind, logger),
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

#[test]
fn matching_with_20_orders() {
    let orders = create_orders();
    let mut book = OrderBook::new();
    let mut log = DummyLogger;
    for order in orders {
        book.execute_order(order, &mut log);
    }
    assert_eq!(book.bid.len(), 3500);
    assert_eq!(book.ask.len(), 3500);

    let order = IncomingOrder {
        price_limit: 10020,
        size: 200,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };
    book.execute_order(order, &mut log);
    assert_eq!(book.bid.len(), 3500);
    assert_eq!(book.ask.len(), 3500-20);
}
