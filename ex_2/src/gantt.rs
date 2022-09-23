use std::{collections::HashMap, time::Duration};

fn format_time(time: &Duration) -> String {
    format!("{:06.3}", time.as_millis() as f32 / 1000.)
}

trait Markdown {
    /// Export to mermaid.js markdown, as a list of rows.
    fn to_md(&self) -> Vec<String>;
}

struct Milestone {
    name: String,
    at: Duration,
}

struct Task {
    name: String,
    start_at: Duration,
    end_at: Duration,
}

enum Record {
    Milestone(Milestone),
    Task(Task),
}

impl Markdown for Record {
    fn to_md(&self) -> Vec<String> {
        let row = match self {
            Record::Milestone(Milestone { name, at }) => {
                format!("{}: milestone, {}, 0", name, format_time(at))
            }
            Record::Task(Task {
                name,
                start_at,
                end_at,
            }) => format!(
                "{}: {}, {}",
                name,
                format_time(start_at),
                format_time(end_at)
            ),
        };
        vec![row]
    }
}

struct Section {
    schedule: Vec<Record>,
}
impl Markdown for Section {
    fn to_md(&self) -> Vec<String> {
        let mut rows = vec![];
        for r in self.schedule.iter() {
            rows.extend(r.to_md());
        }
        rows
    }
}

pub struct Gantt {
    sections: HashMap<String, Section>,
}

impl Gantt {
    pub fn new() -> Gantt {
        Gantt {
            sections: HashMap::new(),
        }
    }

    pub fn push_task(&mut self, section: &str, task: String, start_at: Duration, end_at: Duration) {
        let section = self.sections.entry(section.to_string()).or_insert(Section {
            schedule: Vec::new(),
        });
        section.schedule.push(Record::Task(Task {
            name: task,
            start_at,
            end_at,
        }))
    }

    pub fn push_milestone(&mut self, section: &str, milestone: String, at: Duration) {
        let section = self.sections.entry(section.to_string()).or_insert(Section {
            schedule: Vec::new(),
        });
        section.schedule.push(Record::Milestone(Milestone {
            name: milestone,
            at,
        }))
    }

    pub fn to_md(&self) -> Vec<String> {
        let mut rows = vec![
            "gantt".to_string(),
            "dateFormat ss.SSS".to_string(),
            "axisFormat %S.%L s".to_string(),
        ];

        let mut names: Vec<_> = self.sections.keys().collect();
        names.sort_unstable();

        for name in names.iter() {
            rows.push("".to_string());
            rows.push(format!("section {}", name));
            rows.extend(self.sections.get(*name).unwrap().to_md());
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_gantt() {
        let mut gantt = Gantt::new();
        gantt.push_task(
            "1",
            "task".to_string(),
            Duration::from_secs(0),
            Duration::from_secs(1),
        );
        gantt.push_milestone("2", "task".to_string(), Duration::from_secs(2));

        assert_eq!(
            gantt.to_md(),
            [
                "gantt",
                "dateFormat ss.SSS",
                "axisFormat %S.%L s",
                "",
                "section 1",
                "task: 00.000, 01.000",
                "",
                "section 2",
                "task: milestone, 02.000, 0"
            ]
        );
    }

    #[test]
    fn time_formatting() {
        assert_eq!(format_time(&Duration::new(4, 0)), "04.000");
        assert_eq!(format_time(&Duration::from_millis(1234)), "01.234");
    }
}
