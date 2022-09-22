use super::semaphore::{signal, wait};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq)]
pub enum OperatorRole {
    Reader,
    Writer,
}

#[derive(Debug, PartialEq)]
pub struct Operator {
    /// 序号
    pub id: u32,
    /// 角色
    pub role: OperatorRole,
    /// 操作开始时刻，单位为秒，正数
    pub start_at: f32,
    /// 操作持续时间，正数
    pub duration: f32,
}

pub struct Reporter {
    now: Instant,
}

pub enum Action {
    /// 创建线程
    Create,
    /// 申请操作
    Request,
    /// 开始操作
    Start,
    /// 结束操作
    End,
}

impl Reporter {
    pub fn new() -> Reporter {
        Reporter {
            now: Instant::now(),
        }
    }

    pub fn report(&self, who: &Operator, action: Action) {
        println!(
            "{:6.3} s | #{}：{}。",
            self.now.elapsed().as_millis() as f32 / 1000.,
            who.id,
            match action {
                Action::Create => "🚀创建",
                Action::Request => "🔔申请",
                Action::Start => match who.role {
                    OperatorRole::Reader => "🏁👀开始读取",
                    OperatorRole::Writer => "🏁📝开始写入",
                },
                Action::End => match who.role {
                    OperatorRole::Reader => "🛑👀结束读取",
                    OperatorRole::Writer => "🛑📝结束写入",
                },
            },
        );
    }
}

pub fn run_operators(operators: Vec<Operator>) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));

    let reporter = Arc::new(Reporter::new());

    let mut handles = Vec::new();
    for o in operators {
        let access = Arc::clone(&access);
        let n_readers = Arc::clone(&n_readers);
        let reporter = Arc::clone(&reporter);

        match o.role {
            OperatorRole::Reader => handles.push(thread::spawn(move || {
                reporter.report(&o, Action::Create);

                thread::sleep(Duration::from_secs_f32(o.start_at));

                reporter.report(&o, Action::Request);
                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers += 1;

                    // if I am the first
                    if *n_readers == 1 {
                        wait(&*access);
                    }
                }

                reporter.report(&o, Action::Start);
                thread::sleep(Duration::from_secs_f32(o.duration));
                reporter.report(&o, Action::End);

                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers -= 1;

                    // if I am the last
                    if *n_readers == 0 {
                        signal(&*access);
                    }
                }
            })),
            OperatorRole::Writer => handles.push(thread::spawn(move || {
                reporter.report(&o, Action::Create);

                thread::sleep(Duration::from_secs_f32(o.start_at));

                reporter.report(&o, Action::Request);
                wait(&*access);

                reporter.report(&o, Action::Start);
                thread::sleep(Duration::from_secs_f32(o.duration));
                reporter.report(&o, Action::End);

                signal(&*access);
            })),
        };
    }

    for h in handles {
        h.join().unwrap();
    }
}
