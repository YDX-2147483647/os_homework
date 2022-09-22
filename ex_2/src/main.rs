use std::io;

use ex_2::{run_read_preferring, Operator};

pub fn ready_inputs() -> Vec<Operator> {
    let lines = io::stdin().lines();
    lines
        .map(|l| Operator::from(&l.unwrap()).unwrap())
        .collect()
}

fn main() {
    let operators = ready_inputs();
    run_read_preferring(operators);
}
