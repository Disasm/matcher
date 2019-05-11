//! Logger implementations
use smallvec::SmallVec;

/// Order execution result presented to logger
#[allow(missing_docs)]
#[derive(PartialEq)]
pub enum LogItem {
    /// Order was added to the corresponding order queue
    Enqueued {
        size: u64,
    },
    /// Order was fulfilled with another passive order
    Fulfilled {
        size: u64,
        price: u64,
        user_id: u64,
    },
    /// Order was cancelled
    Cancelled {
        size: u64,
    },
}

impl ToString for LogItem {
    fn to_string(&self) -> String {
        match self {
            LogItem::Enqueued { size } => format!("Q #{}", size),
            LogItem::Fulfilled { size, price, user_id } => format!("F #{} ${} u{}", size, price, user_id),
            LogItem::Cancelled { size } => format!("C #{}", size),
        }
    }
}

/// Represents abstract logger for order execution results
pub trait ExecutionLogger {
    /// Logs execution result
    fn log(&mut self, item: LogItem);

    /// Removes previously logged items
    ///
    /// Used for matching transactions that can be cancelled.
    fn cancel(&mut self);
}

/// Dummy logger which logs everything into the void
pub struct DummyLogger;

impl ExecutionLogger for DummyLogger {
    fn log(&mut self, _item: LogItem) { }

    fn cancel(&mut self) { }
}

/// Logger which uses vector as storage
pub struct VectorLogger(SmallVec<[LogItem; 32]>);

impl VectorLogger {
    /// Constructs `VectorLogger`
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Returns slice representation of logged items
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