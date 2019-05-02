#![allow(unused)]
use matcher::{create_orders, OrderBook, dump20};
use matcher::order::{IncomingOrder, OrderKind, OrderSide};
use matcher::GoodEnoughQueue;

fn main() {
    let orders = create_orders();
    let mut book = OrderBook::new();
    for order in orders {
        book.execute_order(order);
    }
    assert_eq!(book.bid().len(), 3500);
    assert_eq!(book.ask().len(), 3500);

    let order = IncomingOrder {
        price_limit: 10020,
        amount: 200,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };

    //dump20(&book);
    for _ in 0..1000000 {
        book.execute_order(order.clone());

        //dump20(&book);

        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500 - 20);

        for i in 0..20 {
            let order = IncomingOrder {
                price_limit: 10000 + i + 1,
                amount: 10,
                user_id: 100 + i,
                kind: OrderKind::Limit,
                side: OrderSide::Sell
            };
            book.execute_order(order);
        }
        //dump20(&book);
        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500);
    }
}
