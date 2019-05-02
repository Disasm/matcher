use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderKind {
    Limit,
    FillOrKill,
    ImmediateOrCancel,
}

#[derive(Debug, Clone)]
pub struct Order {
    price_limit: u64,
    amount: u64,
    user_id: u64,
    kind: OrderKind,
    side: OrderSide,
}

impl Order {
    fn price_matches(&self, other: &Order) -> bool {
        match other.side {
            OrderSide::Buy => self.price_limit <= other.price_limit,
            OrderSide::Sell => self.price_limit >= other.price_limit,
        }
    }
}

pub struct OrderQueue {
    orders: Vec<Order>,
    side: OrderSide,
}

impl OrderQueue {
    pub fn new(side: OrderSide) -> Self {
        Self {
            orders: Vec::new(),
            side,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Order> {
        self.orders.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Order> {
        self.orders.iter_mut()
    }

    pub fn match_order(&mut self, order: &mut Order) {
        for passive_order in &mut self.orders {
            if passive_order.user_id == order.user_id {
                continue;
            }

            if passive_order.price_matches(order) {
                let amount = cmp::min(order.amount, passive_order.amount);
                passive_order.amount -= amount;
                order.amount -= amount;
                if order.amount == 0 {
                    break;
                }
            } else {
                break;
            }
        }

        // Remove fulfilled passive orders
        self.orders.retain(|order| order.amount > 0);
    }

    pub fn enqueue(&mut self, order: Order) {
        assert_eq!(order.side, self.side);

        match self.side {
            OrderSide::Buy => {
                if let Some(p) = self.orders.iter().position(|o| o.price_limit < order.price_limit) {
                    self.orders.insert(p, order);
                } else {
                    self.orders.push(order);
                }
            },
            OrderSide::Sell => {
                if let Some(p) = self.orders.iter().position(|o| o.price_limit > order.price_limit) {
                    self.orders.insert(p, order);
                } else {
                    self.orders.push(order);
                }
            },
        }
    }
}

pub enum ExecutionResult {
    Enqueued,
    Fulfilled,
    Cancelled,
}

pub struct OrderBook {
    bid: OrderQueue,
    ask: OrderQueue,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bid: OrderQueue::new(OrderSide::Buy),
            ask: OrderQueue::new(OrderSide::Sell),
        }
    }

    pub fn execute_order(&mut self, mut order: Order) {
        match order.side {
            OrderSide::Buy => {
                self.ask.match_order(&mut order);
                if order.amount > 0 {
                    self.bid.enqueue(order);
                }
            }
            OrderSide::Sell => {
                self.bid.match_order(&mut order);
                if order.amount > 0 {
                    self.ask.enqueue(order);
                }
            }
        };
    }

    fn matching_orders(&self, order: &Order) -> usize {
        let queue = match order.side {
            OrderSide::Buy => &self.ask,
            OrderSide::Sell => &self.bid,
        };

        let mut count = 0;
        for passive_order in queue.iter() {
            if passive_order.user_id == order.user_id {
                continue;
            }
            if passive_order.price_matches(order) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

#[allow(unused)]
pub fn create_orders() -> Vec<Order> {
    let price = 10000;
    let mut orders = Vec::new();

    let mut user_id = 10;
    for i in 0..3500 {
        user_id += 1;
        let order = Order {
            price_limit: price + i + 1,
            amount: 10,
            user_id,
            kind: OrderKind::Limit,
            side: OrderSide::Sell
        };
        orders.push(order);
        user_id += 1;
        let order = Order {
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
    assert_eq!(book.bid.orders.len(), 3500);
    assert_eq!(book.ask.orders.len(), 3500);

    let order = Order {
        price_limit: 10020,
        amount: 1000,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };
    book.execute_order(order);
    assert_eq!(book.bid.orders.len(), 3500+1);
    assert_eq!(book.ask.orders.len(), 3500-20);
}
