use std::{io, sync::mpsc};

use arboard::Clipboard;
use clap::{Parser, ValueEnum};

use ex_2::{
    run_read_preferring, run_unspecified_priority, run_write_preferring, Operator, Reporter,
    ReporterConfig,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 策略，例如“读者优先”
    #[clap(arg_enum, value_parser)]
    policy: Policy,
    /// 打印信息时每个进程缩进的数量
    #[clap(short, long, default_value_t = 0, value_parser)]
    tab: u8,
}

/// 策略
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Policy {
    /// 读者优先
    ReadPreferring,
    /// 写者优先
    WritePreferring,
    /// 公平竞争
    UnspecifiedPriority,
}

pub fn ready_inputs() -> Vec<Operator> {
    let lines = io::stdin().lines();
    lines
        .map(|l| Operator::from(&l.unwrap()).unwrap())
        .collect()
}

fn main() {
    let args = Args::parse();
    let config = ReporterConfig { tab: args.tab };

    let operators = ready_inputs();

    let (tx, rx) = mpsc::channel();
    match args.policy {
        Policy::ReadPreferring => run_read_preferring(operators, tx),
        Policy::WritePreferring => run_write_preferring(operators, tx),
        Policy::UnspecifiedPriority => run_unspecified_priority(operators, tx),
    }

    let mut reporter = Reporter::new(config);
    reporter.receive(rx);
    Clipboard::new()
        .unwrap()
        .set_text(reporter.draw().join("\n"))
        .unwrap();
    println!("I've copied to your clipboard. Try to paste it into https://mermaid.live/ .");
}
