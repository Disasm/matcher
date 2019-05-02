use criterion::{criterion_group, criterion_main, BatchSize};
use criterion::Criterion;
use matcher::{create_orders, OrderBook, Order, OrderKind, OrderSide};

#[derive(Clone)]
struct BenchInputData {
    book: OrderBook,
    order: Order,
}

fn execute_order(mut data: BenchInputData) {
    data.book.execute_order(data.order)
}

fn criterion_benchmark(c: &mut Criterion) {
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

    let data = BenchInputData {
        book,
        order,
    };

    c.bench_function("execute order", move |b| b.iter_batched(|| data.clone(), |data| execute_order(data), BatchSize::LargeInput));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
