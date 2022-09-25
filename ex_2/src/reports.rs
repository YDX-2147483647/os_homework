mod gantt;

use std::{collections::HashMap, sync::mpsc::Receiver, time::Duration};

use self::gantt::Gantt;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// 创建线程
    Create,
    /// 申请读取
    RequestRead,
    /// 开始读取
    StartRead,
    /// 结束读取
    EndRead,
    /// 申请写入
    RequestWrite,
    /// 开始写入
    StartWrite,
    /// 结束写入
    EndWrite,
}

pub struct Reporter {
    gantt: Gantt,
    /// Operators that started but not done yet ⇒ start at
    pending_start_at: HashMap<u32, Duration>,
    /// 打印信息时每个进程缩进的数量
    pub tab: u8,
}

/// (who, action, now.elapsed())
pub type ReportMessage = (u32, Action, Duration);

pub struct ReporterConfig {
    /// 打印信息时每个进程缩进的数量
    pub tab: u8,
}

impl Reporter {
    pub fn new(config: ReporterConfig) -> Reporter {
        Reporter {
            gantt: Gantt::new(),
            pending_start_at: HashMap::new(),
            tab: config.tab,
        }
    }

    /// Receive reports
    ///
    /// Reports format: (who, action, now.elapsed())
    pub fn receive(&mut self, rx: Receiver<ReportMessage>) {
        for (who, action, at) in rx {
            let who_str = who.to_string();

            // Update the Gantt
            match action {
                Action::Create => self.gantt.push_milestone(&who_str, "🚀".to_string(), at),
                Action::RequestRead => self.gantt.push_milestone(&who_str, "🔔👀".to_string(), at),
                Action::RequestWrite => self.gantt.push_milestone(&who_str, "🔔📝".to_string(), at),
                Action::StartRead | Action::StartWrite => {
                    let old = self.pending_start_at.insert(who, at);
                    assert!(old.is_none(), "An operator starts again before it ends.");
                }
                Action::EndRead | Action::EndWrite => {
                    let start_at = self
                        .pending_start_at
                        .remove(&who)
                        .expect("An operator ends before it starts.");

                    let task_str = if action == Action::EndRead {
                        "👀"
                    } else {
                        "📝"
                    };

                    self.gantt
                        .push_task(&who_str, task_str.to_string(), start_at, at);
                }
            }

            // Print
            let action_str = match action {
                Action::Create => "🚀创建",
                Action::RequestRead => "🔔👀申请读取",
                Action::RequestWrite => "🔔📝申请写入",
                Action::StartRead => "🏁👀开始读取",
                Action::StartWrite => "🏁📝开始写入",
                Action::EndRead => "🛑👀结束读取",
                Action::EndWrite => "🛑📝结束写入",
            };
            println!(
                "{:6.3} s |{:indent$}#{}：{}。",
                at.as_millis() as f32 / 1000.,
                " ",
                who,
                action_str,
                indent = (who % 8) as usize * self.tab as usize
            );
        }
    }

    pub fn draw(&self) -> Vec<String> {
        self.gantt.to_md()
    }
}
