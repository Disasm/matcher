use std::marker::PhantomData;
use std::cmp::Ordering;
use std::fmt;

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

pub enum TaggedOrder {
    Buy(Order<Buy>),
    Sell(Order<Sell>),
}

impl TaggedOrder {
    pub fn size(&self) -> u64 {
        match self {
            TaggedOrder::Buy(order) => order.size,
            TaggedOrder::Sell(order) => order.size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncomingOrder {
    pub price_limit: u64,
    pub size: u64,
    pub user_id: u64,
    pub kind: OrderKind,
    pub side: OrderSide,
}

impl fmt::Display for IncomingOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let side_letter = match self.side {
            OrderSide::Buy => "B",
            OrderSide::Sell => "S",
        };
        let kind_str = match self.kind {
            OrderKind::Limit => "Lim",
            OrderKind::FillOrKill => "FoK",
            OrderKind::ImmediateOrCancel => "IoC",
        };
        write!(f, "{} {} ${} #{} u{}", kind_str, side_letter, self.price_limit, self.size, self.user_id)
    }
}

impl From<IncomingOrder> for TaggedOrder {
    fn from(order: IncomingOrder) -> Self {
        match order.side {
            OrderSide::Buy => TaggedOrder::Buy(Order {
                price_limit: order.price_limit,
                size: order.size,
                user_id: order.user_id,
                _marker: PhantomData
            }),
            OrderSide::Sell => TaggedOrder::Sell(Order {
                price_limit: order.price_limit,
                size: order.size,
                user_id: order.user_id,
                _marker: PhantomData
            }),
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

    pub fn to_incoming(&self) -> IncomingOrder {
        IncomingOrder {
            price_limit: self.price_limit,
            size: self.size,
            user_id: self.user_id,
            kind: OrderKind::Limit,
            side: D::SIDE,
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
