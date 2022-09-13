from sys import stdin
from pyperclip import copy
from typing import NamedTuple


class Task(NamedTuple):
    id: int
    arrive_at: int
    duration: int
    priority: int
    quantum: int

    def to_md(self) -> str:
        return f"{self.priority}: milestone, {self.arrive_at:03}, 0"


def task_from_raw(raw: str) -> Task:
    return Task(*map(int, raw.split('/')))


class Record(NamedTuple):
    id: int
    start_at: int
    end_at: int
    priority: int

    def to_md(self) -> str:
        return f"{self.priority}: {self.start_at:03}, {self.end_at:03}"


def record_from_raw(raw: str) -> Record:
    # The first is index
    return Record(*map(int, raw.split('/')[1:]))


def to_gantt(tasks: list[Task], schedule: list[Record]) -> list[str]:
    rows: list[str] = [
        'gantt',
        'dateFormat SSS',
        'axisFormat %L ms',
        ''
    ]

    for t in tasks:
        rows.append(f'section {t.id}')
        rows.append(t.to_md())
        for record in schedule:
            if record.id == t.id:
                rows.append(record.to_md())
        rows.append('')

    return rows


def cli():
    input('algorithm id >> ')

    print('tasks >> ')
    tasks: list[Task] = []
    for line in stdin:
        tasks.append(task_from_raw(line))

    print('schedule >> ')
    schedule: list[Record] = []
    for line in stdin:
        schedule.append(record_from_raw(line))

    md = '\n'.join(to_gantt(tasks, schedule))
    copy(md)
    print(md)
    print("I've copied to your clipboard. Try to paste it to https://mermaid.live/ .")


if __name__ == '__main__':
    cli()
