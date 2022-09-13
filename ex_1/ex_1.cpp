#include <assert.h>
#include <iostream>
#include <list>
#include <signal.h>
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

    bool operator==(const Task &other)
    {
        return this->id == other.id;
    }
};

/** 运行了一半的任务，只有可抢占算法会用到。 */
struct TaskRuntime {
    /** 进程号 */
    int id;
    /** 剩余运行时间 */
    int duration_left;
    /** 优先级 */
    int priority;
    /** 时间片 */
    int quantum;

    TaskRuntime(const Task &task)
        : id(task.id), duration_left(task.duration), priority(task.priority), quantum(task.quantum) {}

    bool operator==(const TaskRuntime &other)
    {
        return this->id == other.id;
    }
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
    /** 任务列表，按到达时间升序排列，同时到达时先输入的在前 */
    list<Task> tasks;
};

Input read_input()
{
    Input input;

    int algo_index;
    cin >> algo_index;
    input.algorithm = (Algorithm)algo_index;

    int last_id = -1;
    Task task;
    while (EOF != scanf("%d/%d/%d/%d/%d",
                        &task.id, &task.arrive_at, &task.duration,
                        &task.priority, &task.quantum)) {
        assert(last_id < task.id);

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
    raise(SIGFPE);
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

    int clock = 0;
    auto first_future_task = tasks.begin();
    // arrived but not done tasks
    list<Task> ready_tasks;

    while (first_future_task != tasks.end() || !ready_tasks.empty()) {
        // 1. Update `ready_tasks` and `first_future_task`
        while (first_future_task != tasks.end() && first_future_task->arrive_at <= clock) {
            ready_tasks.push_back(*first_future_task);
            first_future_task++;
        }

        // 2. Find the shortest task in `ready_tasks`
        auto shortest_task = ready_tasks.front();
        for (auto &&t : ready_tasks) {
            if (shortest_task.duration > t.duration) {
                shortest_task = t;
            }
        }
        ready_tasks.remove(shortest_task);

        // 3. Run it
        schedule.push_back(ScheduleRecord(
            shortest_task.id,
            clock,
            clock + shortest_task.duration,
            shortest_task.priority));
        clock += shortest_task.duration;
    }

    return schedule;
}

/**
 * @brief Move arrived tasks to `ready_tasks`
 * If nothing arrives, the clock will be advanced to next arrival, then try again.
 *
 * @param ready_tasks 保证执行后时不空，除非`first_future_task == end`
 * @param first_future_task (can be changed)
 * @param end `first_future_task`所在队列的结尾
 * @param clock (can be changed)
 */
void handle_tasks_arrival(list<TaskRuntime> &ready_tasks, list<Task>::const_iterator &first_future_task, const list<Task>::const_iterator &end, int &clock)
{
    while (first_future_task != end && first_future_task->arrive_at <= clock) {
        ready_tasks.push_back(TaskRuntime(*first_future_task));
        first_future_task++;
    }

    // If nothing arrives and nothing ready, skip to next arrival and try again.
    if (first_future_task != end && ready_tasks.empty()) {
        clock = first_future_task->arrive_at;
        handle_tasks_arrival(ready_tasks, first_future_task, end, clock);
    }
}

Schedule shortest_remaining_time_first(const list<Task> &tasks)
{
    Schedule schedule;

    int clock = 0;
    auto first_future_task = tasks.begin();
    // arrived but not done tasks
    list<TaskRuntime> ready_tasks;

    while (first_future_task != tasks.end() || !ready_tasks.empty()) {
        // 1. Update `ready_tasks` and `first_future_task`
        handle_tasks_arrival(ready_tasks, first_future_task, tasks.end(), clock);

        // 2. Find the shortest task in `ready_tasks`
        auto shortest_task = ready_tasks.front();
        for (auto &&t : ready_tasks) {
            if (shortest_task.duration_left > t.duration_left) {
                shortest_task = t;
            }
        }
        ready_tasks.remove(shortest_task);

        // 3. Run it
        // 3.1 Calculate how long it will run.
        int duration = shortest_task.duration_left;
        if (first_future_task != tasks.end()) {
            duration = min(duration, first_future_task->arrive_at - clock);
        }
        // 3.2 Update the schedule
        if (!schedule.empty() && schedule.back().id == shortest_task.id) {
            // I know it's weird, but the test cases imply this.
            schedule.back().end_at = clock + duration;
        } else {
            schedule.push_back(ScheduleRecord(
                shortest_task.id,
                clock,
                clock + duration,
                shortest_task.priority));
        }
        // 3.3 Let time fly.
        shortest_task.duration_left -= duration;
        clock += duration;

        // 4. Push the task back to `ready_tasks` if not completed
        if (shortest_task.duration_left > 0) {
            ready_tasks.push_back(shortest_task);
        }
    }

    return schedule;
}

Schedule round_robin(const list<Task> &tasks)
{
    Schedule schedule;

    int clock = 0;
    auto first_future_task = tasks.begin();
    // arrived but not done tasks
    list<TaskRuntime> ready_tasks;

    // 0. Update `ready_tasks` and `first_future_task`
    handle_tasks_arrival(ready_tasks, first_future_task, tasks.end(), clock);
    while (first_future_task != tasks.end() || !ready_tasks.empty()) {
        // 2. Get next task in `ready_tasks`
        auto next_task = ready_tasks.front();
        ready_tasks.pop_front();

        // 3. Run it
        // 3.1 Calculate how long it will run.
        int duration = min(next_task.duration_left, next_task.quantum);
        // 3.2 Update the schedule
        schedule.push_back(ScheduleRecord(
            next_task.id,
            clock,
            clock + duration,
            next_task.priority));
        // 3.3 Let time fly.
        next_task.duration_left -= duration;
        clock += duration;

        // 0. Update `ready_tasks` and `first_future_task`
        handle_tasks_arrival(ready_tasks, first_future_task, tasks.end(), clock);

        // 1. Push the task back to `ready_tasks` if not completed
        if (next_task.duration_left > 0) {
            ready_tasks.push_back(next_task);
        }
    }

    return schedule;
}

Schedule dynamic_priority(const list<Task> &tasks)
{
    Schedule schedule;

    int clock = 0;
    auto first_future_task = tasks.begin();
    // arrived but not done tasks
    list<TaskRuntime> ready_tasks;

    const auto end = tasks.end();
    while (first_future_task != end || !ready_tasks.empty()) {
        // 1. Update `ready_tasks` and `first_future_task`
        while (first_future_task != end && first_future_task->arrive_at <= clock) {
            auto new_task = TaskRuntime(*first_future_task);
            if (first_future_task->arrive_at < clock) {
                new_task.priority = max(new_task.priority - 1, 0);
            }
            ready_tasks.push_back(new_task);
            first_future_task++;
        }
        // If nothing arrives and nothing ready, skip to next arrival and try again.
        if (first_future_task != end && ready_tasks.empty()) {
            clock = first_future_task->arrive_at;
            continue;
        }

        // 2. Get next task from `ready_tasks`
        auto next_task = ready_tasks.front();
        for (auto &&t : ready_tasks) {
            if (t.priority < next_task.priority ||
                (t.priority == next_task.priority && t.id < next_task.id)) {
                next_task = t;
            }
        }
        ready_tasks.remove(next_task);

        // 4. Run it
        // 4.0 Update tasks' priorities
        next_task.priority += 3;
        for (auto &&t : ready_tasks) {
            t.priority = max(t.priority - 1, 0);
        }
        // 4.1 Calculate how long it will run.
        int duration = min(next_task.duration_left, next_task.quantum);
        // 4.2 Update the schedule
        schedule.push_back(ScheduleRecord(
            next_task.id,
            clock,
            clock + duration,
            next_task.priority));
        // 4.3 Let time fly.
        next_task.duration_left -= duration;
        clock += duration;

        // 5. Push the task back to `ready_tasks` if not completed
        if (next_task.duration_left > 0) {
            ready_tasks.push_back(next_task);
        }
    }

    return schedule;
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
