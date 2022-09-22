mod operator;
mod semaphore;
mod solutions;

pub use operator::{Action, Operator, OperatorParseError, OperatorRole, Reporter};
pub use solutions::run_read_preferring;
