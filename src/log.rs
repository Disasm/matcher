use smallvec::SmallVec;

pub enum LogItem {
    Enqueued {
        size: u64,
    },
    Fulfilled {
        size: u64,
        price: u64,
        user_id: u64,
    },
    Cancelled {
        size: u64,
    },
}

pub trait ExecutionLogger {
    fn log(&mut self, item: LogItem);

    fn cancel(&mut self);
}

pub struct DummyLogger;

impl ExecutionLogger for DummyLogger {
    fn log(&mut self, _item: LogItem) { }

    fn cancel(&mut self) { }
}

pub struct VectorLogger(SmallVec<[LogItem; 32]>);

impl VectorLogger {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn as_slice(&self) -> &[LogItem] {
        self.0.as_slice()
    }
}

impl ExecutionLogger for VectorLogger {
    fn log(&mut self, item: LogItem) {
        self.0.push(item);
    }

    fn cancel(&mut self) {
        self.0.clear();
    }
}