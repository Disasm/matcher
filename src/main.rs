pub enum OrderSide {
    Buy,
    Sell,
}

pub enum OrderKind {
    Limit,
    FillOrKill,
    ImmediateOrCancel,
}

pub struct Order {
    price_limit: u64,
    amount: u64,
    user_id: u64,
    kind: OrderKind,
    side: OrderSide,
}

pub struct OrderQueue {
    orders: Vec<Order>,
}

impl OrderQueue {
    pub fn new() -> Self {
        Self {
            orders: Vec::new()
        }
    }

    pub fn enqueue(&mut self, order: Order) {
        unimplemented!()
    }
}

pub struct OrderBook {
    bid: OrderQueue,
    ask: OrderQueue,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bid: OrderQueue::new(),
            ask: OrderQueue::new(),
        }
    }

    pub fn match_order(&mut self, order: Order) -> () {
        unimplemented!();
    }
}

fn main() {
    println!("Hello, world!");
}
