#![allow(unused)]
use matcher::{create_orders, OrderBook, dump20};
use matcher::order::{IncomingOrder, OrderKind, OrderSide};
use matcher::GoodEnoughQueue;
use matcher::log::{ExecutionLogger, DummyLogger};

fn main() {
    let orders = create_orders();
    let mut book = OrderBook::deserialize(orders);
    let mut logger = DummyLogger;
    assert_eq!(book.bid().len(), 3500);
    assert_eq!(book.ask().len(), 3500);

    let order = IncomingOrder {
        price_limit: 10020,
        size: 200,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };

    //dump20(&book);
    for _ in 0..1000000 {
        book.execute_order(order.clone(), &mut logger);

        //dump20(&book);

        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500 - 20);

        for i in (0..20).rev() {
            let order = IncomingOrder {
                price_limit: 10000 + i + 1,
                size: 10,
                user_id: 100 + i,
                kind: OrderKind::Limit,
                side: OrderSide::Sell
            };
            book.execute_order(order, &mut logger);
        }
        //dump20(&book);
        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500);
    }
}
