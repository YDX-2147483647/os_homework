use std::{
    sync::{Arc, Mutex},
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

impl Operator {
    pub fn say(&self, words: &str, now: &Instant) {
        println!(
            "{:6.3} s | #{}：{}。",
            now.elapsed().as_millis() as f32 / 1000.,
            self.id,
            words,
        );
    }
}

pub fn run_operators(operators: Vec<Operator>) {
    let access_right = Arc::new(Mutex::new(true));
    let n_readers = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let now = Instant::now();

    for o in operators {
        let n_readers = Arc::clone(&n_readers);
        let access_right = Arc::clone(&access_right);

        match o.role {
            OperatorRole::Reader => {
                o.say("创建", &now);
                handles.push(thread::spawn(move || {
                    thread::sleep(Duration::from_secs_f32(o.start_at));

                    o.say("申请", &now);
                    {
                        let mut n_readers = n_readers.lock().unwrap();
                        *n_readers += 1;

                        // if I am the first
                        if *n_readers == 1 {
                            let mut access_right = access_right.lock().unwrap();
                            *access_right = false;
                        }
                    }

                    o.say("开始读取", &now);
                    thread::sleep(Duration::from_secs_f32(o.duration));
                    o.say("结束读取", &now);

                    {
                        let mut n_readers = n_readers.lock().unwrap();
                        *n_readers += 1;

                        // if I am the last
                        if *n_readers == 0 {
                            let mut access_right = access_right.lock().unwrap();
                            *access_right = true;
                        }
                    }
                }))
            }
            OperatorRole::Writer => {
                todo!("`access_right`现在未实现读写互斥。");
                // o.say("创建", &now);
                // handles.push(thread::spawn(move || {
                //     thread::sleep(Duration::from_secs_f32(o.start_at));

                //     o.say("申请", &now);
                //     let mut access_right = access_right.lock().unwrap();
                //     *access_right = false;

                //     o.say("开始写入", &now);
                //     thread::sleep(Duration::from_secs_f32(o.duration));
                //     o.say("结束写入", &now);

                //     *access_right = true;
                // }))
            }
        };
    }

    for h in handles {
        h.join().unwrap();
    }
}
