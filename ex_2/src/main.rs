use std::io;

use ex_2::{parse_operator, run_operators, Operator};

pub fn ready_inputs() -> Vec<Operator> {
    let lines = io::stdin().lines();
    lines
        .map(|l| parse_operator(&l.unwrap()).unwrap())
        .collect()
}

fn main() {
    let operators = ready_inputs();
    run_operators(operators);
}
