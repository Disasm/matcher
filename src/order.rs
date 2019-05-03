use std::marker::PhantomData;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderKind {
    Limit,
    FillOrKill,
    ImmediateOrCancel,
}

//#[repr(align(128))]
#[derive(Debug, Clone)]
pub struct Order<D> {
    pub price_limit: u64,
    pub size: u64,
    pub user_id: u64,
    _marker: PhantomData<D>,
}

#[derive(Debug, Clone)]
pub struct IncomingOrder {
    pub price_limit: u64,
    pub size: u64,
    pub user_id: u64,
    pub kind: OrderKind,
    pub side: OrderSide,
}

impl From<IncomingOrder> for Order<Buy> {
    fn from(order: IncomingOrder) -> Self {
        assert_eq!(order.side, OrderSide::Buy);
        Self {
            price_limit: order.price_limit,
            size: order.size,
            user_id: order.user_id,
            _marker: PhantomData
        }
    }
}

impl From<IncomingOrder> for Order<Sell> {
    fn from(order: IncomingOrder) -> Self {
        assert_eq!(order.side, OrderSide::Sell);
        Self {
            price_limit: order.price_limit,
            size: order.size,
            user_id: order.user_id,
            _marker: PhantomData
        }
    }
}

pub trait Direction: Clone {
    type Other: Direction;
    const SIDE: OrderSide;
}

#[derive(Clone, Debug)]
pub enum Buy {}

#[derive(Clone, Debug)]
pub enum Sell {}

impl Direction for Buy {
    type Other = Sell;
    const SIDE: OrderSide = OrderSide::Buy;
}

impl Direction for Sell {
    type Other = Buy;
    const SIDE: OrderSide = OrderSide::Sell;
}

impl<D: Direction> Direction for Order<D> {
    type Other = D::Other;
    const SIDE: OrderSide = D::SIDE;
}

impl<D: Direction> Order<D> {
    pub fn price_matches(&self, other: &Order<D::Other>) -> bool {
        match D::SIDE {
            OrderSide::Buy => other.price_limit <= self.price_limit,
            OrderSide::Sell => other.price_limit >= self.price_limit,
        }
    }
}

impl<D: Direction> PartialEq for Order<D> {
    fn eq(&self, other: &Order<D>) -> bool {
        self.price_limit == other.price_limit
    }
}

impl<D: Direction> PartialOrd for Order<D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let order = if self.price_limit == other.price_limit {
            Ordering::Equal
        } else {
            let other_is_better = match D::SIDE {
                OrderSide::Buy => other.price_limit > self.price_limit,
                OrderSide::Sell => other.price_limit < self.price_limit,
            };
            if other_is_better {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        };
        Some(order)
    }
}
