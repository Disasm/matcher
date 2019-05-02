use matcher::{create_orders, OrderBook, Order, OrderKind, OrderSide};

fn main() {
    let orders = create_orders();
    let mut book = OrderBook::new();
    for order in orders {
        book.execute_order(order);
    }

    let order = Order {
        price_limit: 10020,
        amount: 1000,
        user_id: 0,
        kind: OrderKind::Limit,
        side: OrderSide::Buy
    };

    for _ in 0..100000 {
        let mut book = book.clone();
        let order = order.clone();
        book.execute_order(order);
    }
}
