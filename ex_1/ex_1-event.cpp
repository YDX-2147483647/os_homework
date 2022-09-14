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
    bool operator!=(const Task &other)
    {
        return !this->operator==(other);
    }
};

/** 运行了中的任务 */
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
    bool operator!=(const TaskRuntime &other)
    {
        return !this->operator==(other);
    }
};

/** 单条执行记录 */
struct Record {
    /** 进程号 */
    int id;
    /** 开始运行时刻 */
    int start_at;
    /** 结束运行时刻 */
    int end_at;
    /** 优先级 */
    int priority;

    Record(int id, int start_at, int end_at, int priority)
        : id(id), start_at(start_at), end_at(end_at), priority(priority) {}
};

typedef vector<Record> Plan;

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

void print_plan(const Plan &schedule)
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

enum EventType {
    /** [*] → ready */
    Arrive,
    /** running → ready */
    Interrupt,
    /** running → [*] */
    Complete,
};

struct Event {
    EventType type;
};

class Scheduler
{
protected:
    const list<Task> &tasks;

    /** ready and running tasks */
    list<Task> working_tasks;

    /** the running task in `working tasks`, `end` if nothing is running */
    list<Task>::iterator running_task;

    /** events in the future (always ascending sorted) */
    list<Event> events;

public:
    Scheduler(const list<Task> &tasks) : tasks(tasks)
    {
        this->working_tasks = list<Task>();
        this->running_task = this->working_tasks.end();

        this->events = list<Event>();
    }

    Plan run()
    {
        Plan plan = Plan();
        register_arrivals();

        while (!this->events.empty()) {
            auto event = this->events.front();
            this->events.pop_front();

            switch (event.type) {
            case EventType::Arrive:
                this->on_arrive(plan);
                break;
            case EventType::Complete:
                this->on_complete(plan);
                break;
            case EventType::Interrupt:
                this->on_interrupt(plan);
                break;
            }
        }

        return plan;
    }

protected:
    /** Register tasks' arrivals */
    void register_arrivals();

    void on_arrive(Plan &plan);
    void on_complete(Plan &plan);
    void on_interrupt(Plan &plan);
};

int main()
{
    const auto input = read_input();
    assert_sorted(input.tasks);

    auto scheduler = Scheduler(input.tasks);
    const auto plan = scheduler.run();
    print_plan(plan);

    return 0;
}
