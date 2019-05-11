use std::marker::PhantomData;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

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
    pub(crate) price_limit: u64,
    pub(crate) size: u64,
    pub(crate) user_id: u64,
    _marker: PhantomData<D>,
}

pub(crate) enum TaggedOrder {
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
pub struct IncomingOrderParseError;

impl FromStr for IncomingOrder {
    type Err = IncomingOrderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(IncomingOrderParseError);
        }

        let kind = match parts[0] {
            "Lim" => OrderKind::Limit,
            "FoK" => OrderKind::FillOrKill,
            "IoC" => OrderKind::ImmediateOrCancel,
            _ => return Err(IncomingOrderParseError),
        };
        let side = match parts[1] {
            "S" => OrderSide::Sell,
            "B" => OrderSide::Buy,
            _ => return Err(IncomingOrderParseError),
        };

        fn parse_u64_with_prefix(s: &str, prefix: &str) -> Result<u64, IncomingOrderParseError> {
            if s.len() > 1 && s.starts_with(prefix) {
                s[1..].parse().map_err(|_| IncomingOrderParseError)
            } else {
                Err(IncomingOrderParseError)
            }
        }

        let price_limit = parse_u64_with_prefix(parts[2], "$")?;
        let size = parse_u64_with_prefix(parts[3], "#")?;
        let user_id = parse_u64_with_prefix(parts[4], "u")?;

        Ok(IncomingOrder {
            price_limit,
            size,
            user_id,
            kind,
            side,
        })
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

#[test]
fn test_from_str() {
    let order = IncomingOrder::from_str("Lim B $1 #2 u3").unwrap();
    let order2 = IncomingOrder {
        price_limit: 1,
        size: 2,
        user_id: 3,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };
    assert_eq!(order, order2);

    IncomingOrder::from_str("Unk B $1 #2 u3").unwrap_err();
    IncomingOrder::from_str("Lim T $1 #2 u3").unwrap_err();

    IncomingOrder::from_str("Lim B 1 #2 u3").unwrap_err();
    IncomingOrder::from_str("Lim B $$ #2 u3").unwrap_err();
    IncomingOrder::from_str("Lim B $-1 #2 u3").unwrap_err();

    IncomingOrder::from_str("Lim B $1 2 u3").unwrap_err();
    IncomingOrder::from_str("Lim B $1 ## u3").unwrap_err();
    IncomingOrder::from_str("Lim B $1 #-2 u3").unwrap_err();

    IncomingOrder::from_str("Lim B $1 #2 3").unwrap_err();
    IncomingOrder::from_str("Lim B $1 #2 uu").unwrap_err();
    IncomingOrder::from_str("Lim B $1 #2 u-3").unwrap_err();
}
