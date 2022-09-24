use std::{
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

use super::semaphore::{signal, wait};
use crate::{Action, Operator, OperatorRole, Reporter, ReporterConfig};

/// 读者优先方案
///
/// 读者申请时，只要已有其它读者正在读，则它可直接开始操作，不理会写者的请求。
pub fn run_read_preferring(operators: Vec<Operator>, config: ReporterConfig) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));

    let now = Instant::now();

    let (tx, rx) = mpsc::channel();
    for o in operators {
        let access = Arc::clone(&access);
        let n_readers = Arc::clone(&n_readers);
        let tx = tx.clone();

        match o.role {
            OperatorRole::Reader => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestRead, now.elapsed())).unwrap();
                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers += 1;

                    // if I am the first
                    if *n_readers == 1 {
                        wait(&*access);
                    }
                }

                tx.send((o.id, Action::StartRead, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndRead, now.elapsed())).unwrap();

                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers -= 1;

                    // if I am the last
                    if *n_readers == 0 {
                        signal(&*access);
                    }
                }
            }),
            OperatorRole::Writer => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestWrite, now.elapsed()))
                    .unwrap();
                wait(&*access);

                tx.send((o.id, Action::StartWrite, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndWrite, now.elapsed())).unwrap();

                signal(&*access);
            }),
        };
    }

    drop(tx);

    let mut drawer = Reporter::new(config);
    drawer.receive(rx);
    println!("{}", drawer.draw().join("\n"));
}

/// 写者优先方案
///
/// 一旦有写者申请，任何新读者都必须先等待。
pub fn run_write_preferring(operators: Vec<Operator>, config: ReporterConfig) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));
    let n_writers = Arc::new(Mutex::new(0));

    let can_reader_acquire = Arc::new((Mutex::new(true), Condvar::new()));

    let now = Instant::now();

    let (tx, rx) = mpsc::channel();
    for o in operators {
        let access = access.clone();
        let n_readers = n_readers.clone();
        let n_writers = n_writers.clone();
        let can_reader_acquire = can_reader_acquire.clone();
        let tx = tx.clone();

        match o.role {
            OperatorRole::Reader => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestRead, now.elapsed())).unwrap();
                wait(&*can_reader_acquire);
                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers += 1;

                    // if I am the first
                    if *n_readers == 1 {
                        wait(&*access);
                    }
                }
                signal(&*can_reader_acquire);

                tx.send((o.id, Action::StartRead, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndRead, now.elapsed())).unwrap();

                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers -= 1;

                    // if I am the last
                    if *n_readers == 0 {
                        signal(&*access);
                    }
                }
            }),
            OperatorRole::Writer => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestWrite, now.elapsed()))
                    .unwrap();
                {
                    let mut n_writers = n_writers.lock().unwrap();
                    *n_writers += 1;

                    // if I am the first
                    if *n_writers == 1 {
                        wait(&*can_reader_acquire);
                    }
                }

                wait(&*access);
                tx.send((o.id, Action::StartWrite, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndWrite, now.elapsed())).unwrap();
                signal(&*access);

                {
                    let mut n_writers = n_writers.lock().unwrap();
                    *n_writers -= 1;

                    // if I am the last
                    if *n_writers == 0 {
                        signal(&*can_reader_acquire);
                    }
                }
            }),
        };
    }

    drop(tx);

    let mut drawer = Reporter::new(config);
    drawer.receive(rx);
    println!("{}", drawer.draw().join("\n"));
}

/// 公平竞争方案
///
/// This solves the [third readers-writers problem](https://en.wikipedia.org/wiki/Readers%E2%80%93writers_problem#Third_readers%E2%80%93writers_problem).
///
/// 所有操作者都要在`service`的等待队列中排队，从而保证公平。
pub fn run_unspecified_priority(operators: Vec<Operator>, config: ReporterConfig) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let service = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));

    let now = Instant::now();

    let (tx, rx) = mpsc::channel();
    for o in operators {
        let access = access.clone();
        let service = service.clone();
        let n_readers = n_readers.clone();
        let tx = tx.clone();

        match o.role {
            OperatorRole::Reader => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestRead, now.elapsed())).unwrap();
                wait(&*service);
                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers += 1;

                    // if I am the first
                    if *n_readers == 1 {
                        wait(&*access);
                    }
                }
                signal(&*service);

                tx.send((o.id, Action::StartRead, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndRead, now.elapsed())).unwrap();

                {
                    let mut n_readers = n_readers.lock().unwrap();
                    *n_readers -= 1;

                    // if I am the last
                    if *n_readers == 0 {
                        signal(&*access);
                    }
                }
            }),
            OperatorRole::Writer => thread::spawn(move || {
                tx.send((o.id, Action::Create, now.elapsed())).unwrap();

                thread::sleep(Duration::from_secs_f32(o.start_at));

                tx.send((o.id, Action::RequestWrite, now.elapsed()))
                    .unwrap();
                wait(&*service);
                wait(&*access);
                signal(&*service);

                tx.send((o.id, Action::StartWrite, now.elapsed())).unwrap();
                thread::sleep(Duration::from_secs_f32(o.duration));
                tx.send((o.id, Action::EndWrite, now.elapsed())).unwrap();

                signal(&*access);
            }),
        };
    }

    drop(tx);

    let mut drawer = Reporter::new(config);
    drawer.receive(rx);
    println!("{}", drawer.draw().join("\n"));
}
