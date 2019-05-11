use matcher::{create_orders, OrderBook};
use matcher::order::{IncomingOrder, OrderKind, OrderSide};
use matcher::GoodEnoughQueue;
use matcher::log::DummyLogger;

fn main() {
    let orders = create_orders();
    let mut book = OrderBook::from_vec(orders);
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

    let mut reset_orders = Vec::new();
    for order in book.ask() {
        reset_orders.insert(0, order.to_incoming());
        if reset_orders.len() == 20 {
            break;
        }
    }

    for _ in 0..1000000 {
        book.execute_order(order.clone(), &mut logger);

        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500 - 20);

        let mut logger = DummyLogger;
        for order in &reset_orders {
            book.execute_order(order.clone(), &mut logger);
        }

        assert_eq!(book.bid().len(), 3500);
        assert_eq!(book.ask().len(), 3500);
    }
}
