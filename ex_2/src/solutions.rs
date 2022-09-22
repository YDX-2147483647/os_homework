use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use super::semaphore::{signal, wait};
use crate::{Action, Operator, OperatorRole, Reporter, ReporterConfig};

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
