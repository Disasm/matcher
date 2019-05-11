use criterion::{criterion_group, criterion_main, BatchSize};
use criterion::Criterion;
use matcher::{create_orders, OrderBook, GoodEnoughQueue};
use matcher::order::{IncomingOrder, OrderKind, OrderSide};
use matcher::log::DummyLogger;
use std::rc::Rc;
use std::sync::RwLock;

struct BenchInputData {
    shared_book: Rc<RwLock<OrderBook>>,
    order: IncomingOrder,
}

fn reset_book(book: &mut OrderBook, reset_orders: &[IncomingOrder]) {
    assert_eq!(book.bid().len(), 3500);
    if book.ask().len() == 3500 {
        return;
    }
    assert_eq!(book.ask().len(), 3500 - 20);
    let mut logger = DummyLogger;
    for order in reset_orders {
        book.execute_order(order.clone(), &mut logger);
    }
    assert_eq!(book.ask().len(), 3500);
}

impl BenchInputData {
    pub fn new(shared_book: Rc<RwLock<OrderBook>>, incoming_order: &IncomingOrder, reset_orders: &[IncomingOrder]) -> Self {
        let mut book = shared_book.write().unwrap();
        reset_book(&mut book, reset_orders);

        Self {
            shared_book: shared_book.clone(),
            order: incoming_order.clone(),
        }
    }
}

fn execute_order(data: BenchInputData) {
    let mut logger = DummyLogger;
    let mut book = data.shared_book.write().unwrap();
    book.execute_order(data.order, &mut logger);
}

fn criterion_benchmark(c: &mut Criterion) {
    let orders = create_orders();
    let book = OrderBook::from_vec(orders);

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
    let shared_book = Rc::new(RwLock::new(book));

    c.bench_function("execute order", move |b| b.iter_batched(
        || BenchInputData::new(shared_book.clone(), &order, &reset_orders),
        execute_order,
        BatchSize::PerIteration)
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
