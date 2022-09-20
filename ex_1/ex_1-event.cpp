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

        // Find the first task after last arrival
        // or the first task arrives in the meantime but has lower priority.
        auto t = input.tasks.begin();
        const auto end = input.tasks.end();
        while (t != end &&
               (t->arrive_at < task.arrive_at ||
                (t->arrive_at == task.arrive_at && t->priority <= task.priority))) {
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

    PrivateUse,
};

#define NOT_APPLICABLE -1

struct Event {
    EventType type;
    int at;
    /** Only for arrive events, `NOT_APPLICABLE` otherwise */
    int task_id;

    Event(EventType type, int at, int task_id) : type(type), at(at), task_id(task_id) {}
};

using TaskRuntimeIterator = list<TaskRuntime>::iterator;

class Scheduler
{
protected:
    const list<Task> &tasks;

    /** ready and running tasks (default: ascending sort by `arrive_at`) */
    list<TaskRuntime> working_tasks;

    /** the running task in `working tasks`, `end` if nothing is running */
    TaskRuntimeIterator running_task;

    /** events in the future (always ascending sorted) */
    list<Event> events;

public:
    Scheduler(const list<Task> &tasks) : tasks(tasks)
    {
        this->working_tasks = list<TaskRuntime>();
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

            handle_event(event, plan);
        }

        return plan;
    }

protected:
    /**
     * Get the task in `tasks` by id and convert to `TaskRuntime`
     */
    TaskRuntime get_task(int id)
    {
        auto task = this->tasks.begin();

        const auto end = this->tasks.end();
        while (task != end && task->id != id) {
            task++;
        }
        assert(task != end);
        return TaskRuntime(*task);
    }

    virtual void register_event(Event event)
    {
        auto e = this->events.begin();
        const auto end = this->events.end();
        while (e != end && e->at <= event.at) {
            e++;
        }

        this->events.insert(e, event);
    }

    /**
     * @brief Register tasks' arrivals
     *
     * Default implementation: Push tasks by the same order as `this->tasks`.
     */
    virtual void register_arrivals()
    {
        for (auto &&t : this->tasks) {
            // We push back here in favor of using O(n) `register event`.
            this->events.push_back(Event(
                EventType::Arrive,
                t.arrive_at,
                t.id));
        }
    };

    bool nothing_running()
    {
        return this->running_task == this->working_tasks.end();
    }

    virtual void handle_event(Event event, Plan &plan)
    {
        switch (event.type) {
        case EventType::Arrive:
            this->on_arrive(event, plan);
            break;
        case EventType::Complete:
            this->on_complete(event, plan);
            break;
        case EventType::Interrupt:
            this->on_interrupt(event, plan);
            break;
        }
    }

    virtual void on_arrive(Event event, Plan &plan)
    {
        auto task = this->get_task(event.task_id);
        this->working_tasks.push_back(task);

        if (this->nothing_running()) {
            this->on_interrupt(event, plan);
        }
    }

    virtual void on_complete(Event event, Plan &plan)
    {
        this->working_tasks.erase(this->running_task);
        this->running_task = this->working_tasks.end();

        this->on_interrupt(event, plan);
    }

    virtual void on_interrupt(Event event, Plan &plan)
    {
        if (this->working_tasks.empty()) {
            this->running_task = this->working_tasks.end();
            return;
        }

        auto task = this->working_tasks.begin();
        this->running_task = task;
        auto end_at = event.at + task->duration_left;
        plan.push_back(Record(task->id, event.at, end_at, task->priority));

        this->register_event(Event(EventType::Complete, end_at, NOT_APPLICABLE));
    };
};

class SchedulerFCFS : public Scheduler
{
public:
    SchedulerFCFS(const list<Task> &tasks) : Scheduler(tasks) {}
};

class SchedulerSJF : public Scheduler
{
public:
    SchedulerSJF(const list<Task> &tasks) : Scheduler(tasks) {}

protected:
    void on_arrive(Event event, Plan &plan)
    {
        auto task = this->get_task(event.task_id);

        // We sort `working_tasks` by `duration_left`
        auto where = this->working_tasks.begin();
        const auto end = this->working_tasks.end();
        while (where != end && where->duration_left <= task.duration_left) {
            ++where;
        }

        this->working_tasks.insert(where, task);

        if (this->nothing_running()) {
            this->on_interrupt(event, plan);
        }
    }
};

class SchedulerPreemptive : public Scheduler
{
public:
    SchedulerPreemptive(const list<Task> &tasks) : Scheduler(tasks) {}

protected:
    virtual void on_interrupt(Event event, Plan &plan)
    {
        this->handle_last_running_task();

        if (this->working_tasks.empty()) {
            return;
        }
        this->running_task = this->next_task_to_run();

        const auto duration = this->can_run_for(event.at);
        this->running_task->duration_left -= duration;

        auto end_at = event.at + duration;
        this->record_running_task(plan, event.at, end_at);

        if (this->running_task->duration_left > 0) {
            this->register_event(Event(EventType::Interrupt, end_at, NOT_APPLICABLE));
        } else {
            this->register_event(Event(EventType::Complete, end_at, NOT_APPLICABLE));
        }
    }

    virtual void handle_last_running_task()
    {
        this->running_task = this->working_tasks.end();
    }

    virtual void record_running_task(Plan &plan, int start_at, int end_at)
    {
        plan.push_back(Record(this->running_task->id, start_at, end_at, this->running_task->priority));
    }

    virtual TaskRuntimeIterator next_task_to_run()
    {
        return this->working_tasks.begin();
    };

    /** how long can the `running_task` run for from `now` */
    virtual int can_run_for(int now)
    {
        return this->running_task->duration_left;
    }
};

class SchedulerShortestRemainingTimeFirst : public SchedulerPreemptive
{
public:
    SchedulerShortestRemainingTimeFirst(const list<Task> &tasks) : SchedulerPreemptive(tasks) {}

protected:
    TaskRuntimeIterator next_task_to_run()
    {
        auto task = this->working_tasks.begin();

        const auto end = this->working_tasks.end();
        for (auto t = this->working_tasks.begin(); t != end; ++t) {
            if (t->duration_left < task->duration_left) {
                task = t;
            }
        }

        return task;
    }

    int can_run_for(int now)
    {
        auto next_arrival = this->events.begin();
        const auto end = this->events.end();
        while (next_arrival != end && next_arrival->type != EventType::Arrive) {
            ++next_arrival;
        }

        if (next_arrival == end) {
            // if nothing will arrive
            return this->running_task->duration_left;
        } else {
            return min(this->running_task->duration_left, next_arrival->at - now);
        }
    }

    void record_running_task(Plan &plan, int start_at, int end_at)
    {
        if (!plan.empty() && this->running_task->id == plan.back().id) {
            plan.back().end_at = end_at;
        } else {
            SchedulerPreemptive::record_running_task(plan, start_at, end_at);
        }
    }
};

class SchedulerRoundRobin : public SchedulerPreemptive
{
public:
    SchedulerRoundRobin(const list<Task> &tasks) : SchedulerPreemptive(tasks) {}

protected:
    int can_run_for(int now)
    {
        return min(this->running_task->duration_left, this->running_task->quantum);
    }

    void handle_last_running_task()
    {
        if (this->running_task != this->working_tasks.end()) {
            // Move last `running_task` to the end
            this->working_tasks.splice(this->working_tasks.end(), this->working_tasks, this->running_task);
        }

        this->running_task = this->working_tasks.end();
    }
};

/**
 * @note
 *
 * Timeline:
 *
 * 1. A task starts running.
 * 2. Some tasks arrive. (Their priorities -= 1)
 * 3. The running task is interrupted.
 * 4. Some other tasks arrives in the meantime. (Their priorites don't change)
 * 5. We decide what to run next.
 *
 * Ideally, We update priorities between 3 and 4.
 *
 * But it's impossible: arrive events are handled before the interrupt event.
 * In other words, 4 will happen before 3. Therefore, we introduce a new event to
 * increase priorities.
 */
class SchedulerDynamicPriority : public SchedulerPreemptive
{
public:
    SchedulerDynamicPriority(const list<Task> &tasks) : SchedulerPreemptive(tasks) {}

protected:
    TaskRuntimeIterator next_task_to_run()
    {
        auto task = this->working_tasks.begin();

        const auto end = this->working_tasks.end();
        for (auto t = this->working_tasks.begin(); t != end; ++t) {
            if (t->priority < task->priority) {
                task = t;
            }
        }

        return task;
    }

    int can_run_for(int now)
    {
        return min(this->running_task->duration_left, this->running_task->quantum);
    }

    /** Decrease the `running_task`'s priority then record it */
    void record_running_task(Plan &plan, int start_at, int end_at)
    {
        this->running_task->priority += 3;

        SchedulerPreemptive::record_running_task(plan, start_at, end_at);
    }

    void register_event(Event event)
    {
        if (event.type == EventType::Complete || event.type == EventType::Interrupt) {
            auto e = this->events.begin();
            const auto end = this->events.end();
            while (e != end && e->at < event.at) {
                e++;
            }

            this->events.insert(e, Event(EventType::PrivateUse, event.at, NOT_APPLICABLE));
        }

        SchedulerPreemptive::register_event(event);
    }

    void handle_event(Event event, Plan &plan)
    {
        if (event.type == EventType::PrivateUse) {
            // Increase ready tasks' priorites
            for (auto &&t : this->working_tasks) {
                if (t != *this->running_task) {
                    t.priority = max(t.priority - 1, 0);
                }
            }
        } else {
            SchedulerPreemptive::handle_event(event, plan);
        }
    }
};

int main()
{
    const auto input = read_input();
    assert_sorted(input.tasks);

    Scheduler *scheduler = NULL;
    switch (input.algorithm) {
    case Algorithm::FirstComeFirstService:
        scheduler = new SchedulerFCFS(input.tasks);
        break;
    case Algorithm::ShortestJobFirst:
        scheduler = new SchedulerSJF(input.tasks);
        break;
    case Algorithm::ShortestRemainingTimeFirst:
        scheduler = new SchedulerShortestRemainingTimeFirst(input.tasks);
        break;
    case Algorithm::RoundRobin:
        scheduler = new SchedulerRoundRobin(input.tasks);
        break;
    case Algorithm::DynamicPriority:
        scheduler = new SchedulerDynamicPriority(input.tasks);
        break;

    default:
        not_implemented();
        break;
    }

    print_plan(scheduler->run());
    delete scheduler;

    return 0;
}
