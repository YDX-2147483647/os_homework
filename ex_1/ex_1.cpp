#include <assert.h>
#include <iostream>
#include <list>
#include <string.h>
#include <vector>

using namespace std;

/** 调度算法 */
enum Algorithm {
    /** 先来先服务 */
    FirstComeFirstService = 1,
    /** 短作业优先 */
    ShortestJobFirst = 2,
    /** 最短剩余时间优先 */
    ShortestRemainingTimeFirst = 3,
    /** 时间片轮转 */
    RoundRobin = 4,
    /** 动态优先级 */
    DynamicPriority = 5,
};

/** 任务 */
struct Task {
    /** 进程号 */
    int id;
    /** 到达时刻 */
    int arrive_at;
    /** 运行时间 */
    int duration;
    /** 优先级 */
    int priority;
    /** 时间片 */
    int quantum;
};

/** 单条执行记录 */
struct ScheduleRecord {
    /** 进程号 */
    int id;
    /** 开始运行时刻 */
    int start_at;
    /** 结束运行时刻 */
    int end_at;
    /** 优先级 */
    int priority;

    ScheduleRecord(int id, int start_at, int end_at, int priority)
        : id(id), start_at(start_at), end_at(end_at), priority(priority) {}
};

typedef vector<ScheduleRecord> Schedule;

/** 输入 */
struct Input {
    Algorithm algorithm;
    /** 任务列表，按到达时间升序排列 */
    list<Task> tasks;
};

Input read_input()
{
    Input input;

    int algo_index;
    cin >> algo_index;
    input.algorithm = (Algorithm)algo_index;

    Task task;
    while (EOF != scanf("%d/%d/%d/%d/%d",
                        &task.id, &task.arrive_at, &task.duration,
                        &task.priority, &task.quantum)) {
        // Find the first task after last arrival.
        auto t = input.tasks.begin();
        const auto end = input.tasks.end();
        while (t != end && t->arrive_at <= task.arrive_at) {
            t++;
        }
        // Insert
        input.tasks.insert(t, Task(task));
    }

    return input;
}

void print_schedule(const Schedule &schedule)
{
    int index = 1;
    for (const auto &record : schedule) {
        printf("%d/%d/%d/%d/%d\n",
               index,
               record.id, record.start_at, record.end_at, record.priority);
        index++;
    }
}

void assert_sorted(const list<Task> &tasks)
{
    int last_arrive_at = -1;
    for (auto &&t : tasks) {
        assert(t.arrive_at >= last_arrive_at);
        last_arrive_at = t.arrive_at;
    }
}

void not_implemented()
{
    cerr << "Not implemented." << endl;
    assert(false);
}

Schedule first_come_first_service(const list<Task> &tasks)
{
    Schedule schedule;

    int clock = 0;
    for (auto &&t : tasks) {
        if (clock < t.arrive_at) {
            clock = t.arrive_at;
        }

        schedule.push_back(
            ScheduleRecord(
                t.id,
                clock,
                clock + t.duration,
                t.priority));

        clock += t.duration;
    }

    return schedule;
}

Schedule shortest_job_first(const list<Task> &tasks)
{
    Schedule schedule;
    // todo
    return schedule;
}

Schedule shortest_remaining_time_first(const list<Task> &tasks)
{
    not_implemented();
}

Schedule round_robin(const list<Task> &tasks)
{
    not_implemented();
}

Schedule dynamic_priority(const list<Task> &tasks)
{
    not_implemented();
}

int main()
{
    const auto input = read_input();
    assert_sorted(input.tasks);

    switch (input.algorithm) {
    case Algorithm::FirstComeFirstService:
        print_schedule(first_come_first_service(input.tasks));
        break;
    case Algorithm::ShortestJobFirst:
        print_schedule(shortest_job_first(input.tasks));
        break;
    case Algorithm::ShortestRemainingTimeFirst:
        print_schedule(shortest_remaining_time_first(input.tasks));
        break;
    case Algorithm::RoundRobin:
        print_schedule(round_robin(input.tasks));
        break;
    case Algorithm::DynamicPriority:
        print_schedule(dynamic_priority(input.tasks));
        break;

    default:
        not_implemented();
        break;
    }

    return 0;
}
