use std::sync::{Condvar, Mutex};

type Semaphore = (Mutex<bool>, Condvar);

pub fn wait(semaphore: &Semaphore) {
    let (lock, cvar) = semaphore;
    let mut lock = lock.lock().unwrap();
    while !*lock {
        lock = cvar.wait(lock).unwrap();
    }
    *lock = false;
}

pub fn signal(semaphore: &Semaphore) {
    let (lock, cvar) = semaphore;
    let mut lock = lock.lock().unwrap();
    *lock = true;
    cvar.notify_one();
}
