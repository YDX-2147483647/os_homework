use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use super::semaphore::{signal, wait};
use crate::{Action, Operator, OperatorRole, Reporter, ReporterConfig};

/// 读者优先方案
///
/// 读者申请时，只要已有其它读者正在读，则它可直接开始操作，不理会写者的请求。
pub fn run_read_preferring(operators: Vec<Operator>, config: ReporterConfig) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));

    let reporter = Arc::new(Reporter::new(config));

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

/// 写者优先方案
///
/// 一旦有写者申请，任何新读者都必须先等待。
pub fn run_write_preferring(operators: Vec<Operator>, config: ReporterConfig) {
    let access = Arc::new((Mutex::new(true), Condvar::new()));
    let n_readers = Arc::new(Mutex::new(0));
    let n_writers = Arc::new(Mutex::new(0));

    let can_reader_acquire = Arc::new((Mutex::new(true), Condvar::new()));

    let reporter = Arc::new(Reporter::new(config));

    let mut handles = Vec::new();
    for o in operators {
        let access = access.clone();
        let n_readers = n_readers.clone();
        let n_writers = n_writers.clone();
        let can_reader_acquire = can_reader_acquire.clone();
        let reporter = reporter.clone();

        match o.role {
            OperatorRole::Reader => handles.push(thread::spawn(move || {
                reporter.report(&o, Action::Create);

                thread::sleep(Duration::from_secs_f32(o.start_at));

                reporter.report(&o, Action::Request);
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
                {
                    let mut n_writers = n_writers.lock().unwrap();
                    *n_writers += 1;

                    // if I am the first
                    if *n_writers == 1 {
                        wait(&*can_reader_acquire);
                    }
                }

                wait(&*access);
                reporter.report(&o, Action::Start);
                thread::sleep(Duration::from_secs_f32(o.duration));
                reporter.report(&o, Action::End);
                signal(&*access);

                {
                    let mut n_writers = n_writers.lock().unwrap();
                    *n_writers -= 1;

                    // if I am the last
                    if *n_writers == 0 {
                        signal(&*can_reader_acquire);
                    }
                }
            })),
        };
    }

    for h in handles {
        h.join().unwrap();
    }
}
