# 实验1 单处理机进程调度

## 实验内容

编写单处理机下的进程调度程序，模拟操作系统对进程的调度。

## 实验目的

进程是操作系统中最基本、最重要的概念，进程调度又是操作系统的核心模块。

本实验要求学生独立设计并实现进程调度模拟程序，以加深对进程控制块概念和各种进程调度算法的理解。

## 实验基础知识

- **进程调度**

  对于单核单处理器系统，同时只能有一个进程运行，其它进程必须等到处理器再次空闲，重新调度时才有可能运行。为了充分利用 CPU、缩短进程等待时间、尽快响应新进程，需要特别设计进程调度。

- 典型的进程是 CPU 与输入输出请求交替执行，阻塞（等待输入输出等事件）时 CPU 可以先让其它进程运行。本实验不涉及任何输入输出请求，所有进程都只含 CPU 计算。

- **进程状态**

  ```mermaid
  stateDiagram-v2
  direction LR
  
  就绪: 就绪<br>ready
  运行: 运行<br>running
  阻塞: 阻塞<br>blocked
  
  [*] --> 就绪: 创建
  运行 --> 就绪: 中断
  就绪 --> 运行: 调度
  运行 --> 阻塞: 等待事件
  阻塞 --> 就绪: 事件完成
  运行 --> [*]: 终止
  ```

- **调度时机**

  只有以下四种情况有可能发生调度，并且前两种必定发生调度。

  - 当前进程发起输入输出请求，或者开始等待别的事件。（运行→阻塞）
  - 当前进程终止。（运行→）
  - 当前进程中断。（运行→就绪）
  - 某个进程就绪。（阻塞→就绪）

  非抢占式方案只在前两种情况调度；抢占（preemptive）式方案则四种都有可能。

  本实验中，“先来先服务”“短作业优先”属于前者，“最短剩余时间”“时间片轮转”“动态优先级”属于后者。另外由于本实验不存在阻塞，非抢占式方案意味着进程一旦开始运行，便会一次运行完。

## 实验设计方法

> 我先写了一个程序；最后发现大段重复，就用 C++ 的各种特性精简合并，重新写了一个，然而代码反而多了一百多行……（语法太繁琐了，`public`之类的就独占十几行。）
>
> 两个程序都正确，思路不太一样，下面分开依次介绍。

### 总体设计

#### 结构

首先针对输入输出设计结构。

> 以下如无说明，皆为`struct`。

- **输入方面**
  
  - 调度算法`Algorithm`。（枚举）
  
  - 任务`Task`：输入的一行。
  
  - 任务列表`list<Task>`：输入的任务列表。
  
    因为长度不定，之后又有可能频繁增删，选用链表`list`。
  
  - 输入`Input`。
  
- **输出方面**
  - 单条执行记录`ScheduleRecord`：输出的一行。
  
  - 执行记录`Schedule`：输出。
  
    因为记录只增不减，可用`vector`。（`Schedule`是`vector<ScheduleRecord>`的别名。）`Schedule`并不显式存储编号，而是用存储位置关系隐含编号。

```mermaid
classDiagram

class Algorithm

class Task {
    int id 进程号
    int arrive_at 到达时刻
    int duration 运行时间
    int priority 优先级
    int quantum 时间片
}

class Input {
    Algorithm algorithm 算法
    list~Task~ tasks 任务列表
}

Algorithm --> Input
Task ..> Input

class ScheduleRecord {
    int id 进程号
    int start_at 开始运行时刻
    int end_at 结束运行时刻
    int priority 优先级
}

ScheduleRecord ..> Schedule
```

```c++
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
```

> `○○_at`表示时刻，`duration_○○`表示时间。

在抢占式方案中，任务可能分好几次运行。此时我们只关心“剩余”运行时间（而非“总”运行时间），任务到达时刻也不再重要。因此，另外设计一个`TaskRuntime`记录运行了一半的任务。

```mermaid
classDiagram

class Task {
    int id 进程号
    int arrive_at 到达时刻
    int duration 运行时间
    int priority 优先级
    int quantum 时间片
}

class TaskRuntime {
    int id 进程号
    int duration_left 剩余运行时间
    int priority 优先级
    int quantum 时间片

    TaskRuntime(const Task &task)
}

```

#### 功能和流程图

```mermaid
flowchart LR
read_input -->|input| 匹配要求的方案 -->|input.tasks| 相应方案 -->|schedule| print_schedule
```

1. **`read_input`**

   1. 从`stdin`读取输入并解析。
   2. 给`tasks`排序：==按到达时间升序排列，同时到达时进程号小的在前。==

   注：实际是边解析，边插入排序，分得没那么清。另外，由于所有测试输入进程号都是从小到大，所以实际用“先输入的在前”代替了“进程号小的在前”。

2. **匹配要求的方案**

   用`switch`–`case`即可。

   如果输入非法，调用`not_implemented()`向`stderr`报错并结束程序。

3. **相应方案**

   ==每种方案一个函数==，将任务列表（`const list<Task> &tasks`）转换为执行计划（`Schedule`）。

   注意`tasks`是`const`，本身并不更改。

4. **`print_schedule`**

   按顺序自动编号，向`stdout`输出。

下面依次介绍五种方案的函数。

### 调度方案设计

都采用时钟（`int clock`）从零开始模拟运行。

#### 1. 先来先服务`first_come_first_service`

```mermaid
flowchart TB
new[新建空 schedule] --> zero[int clock = 0] --> for --> return[返回 schedule]

subgraph for[用 t 顺序遍历 tasks]
    should_skip{"t 到了？"} -->|到了| push["记录<br>schedule.push_back(…)"] --> fly[时间流逝<br>clock += t.duration]
    should_skip -->|没到| skip[调表快进<br>clock = t.arrive_at] --> push
end
```

`clock`的用处在于记录执行计划，如下。

```c++
schedule.push_back(
    ScheduleRecord(
        t.id,
        clock,
        clock + t.duration,
        t.priority));
```

#### 2. 短作业优先`shortest_job_first`

这种方案中，任务状态变化过程如下。

```mermaid
flowchart LR
future["未到达<br>future"]
--> ready["就绪<br>ready"]
--> running["运行<br>running"]
--> done["完成<br>done"]

style running stroke:#f66,stroke-width:2px
style done fill:orange, stroke-dasharray: 5 5
```

1. **未到达**：由于`tasks`已按到达顺序排好，用`first_future_task`标记首个未到达的任务即可。
2. **未到达→就绪**：任务到达时，将它复制一下插入`ready_tasks`。
3. **就绪→运行**：轮到它运行时，将它从`ready_tasks`中删除，运行之。
4. **运行→完成**：记录一下，直接丢弃。

```mermaid
flowchart LR

classDef running stroke:#f66, stroke-width:2px
classDef done fill:orange, stroke-dasharray: 5 5

subgraph tasks["<code>tasks</code>"]
    direction LR
    t1[" "] --- t2[" "] --- t3[" "] --- t4[" "] --- t5[" "]
end

r1[" "]:::done --- r2[" "]:::running --- r3

subgraph ready_tasks["<code>ready_tasks</code>"]
    direction LR
    r3[" "] --- r4[" "]
end

t1 -.- r1
t2 -.- r2
t3 -.- r3
t4 -.- r4

first_future(["<code>first_future_task</code>"]) --> t5
style first_future fill:#fff0, stroke:#fff0;
```

程序流程如下。

> 橙色部分表示与前面不同，后同。

```mermaid
flowchart TB
subgraph init[初始化]
    direction TB
    new_schedule[新建空 schedule]
    new_ready_tasks[新建空 ready_tasks]:::diff
    init_first_future_task["first_future_task = tasks.begin()"]:::diff
    zero[int clock = 0]
end

init --> while --> return[返回 schedule]

subgraph while["一直循环，直到没有未到达或就绪的任务"]
    必要时调表快进  --> 更新新到达任务的状态:::diff --> select[从 ready_tasks 中挑出运行时间最短的]:::diff --> 记录并让时间流逝
end

classDef diff fill: orange;
```

- **一直循环，直到没有未到达或就绪的任务**：

  不存在“未到达”或“就绪” ⇔ 全都运行过。

- **更新新到达任务的状态**：

  ```c++
  while (first_future_task != tasks.end() && first_future_task->arrive_at <= clock) {
      ready_tasks.push_back(*first_future_task);
      first_future_task++;
  }
  ```

- **从`ready_tasks`中挑出运行时间最短的**：

  1. 擂台法找最小，相同时选靠前的（先来先服务）。

     ```c++
     auto shortest_task = ready_tasks.front();
     for (auto &&t : ready_tasks) {
         if (shortest_task.duration > t.duration) {
             shortest_task = t;
         }
     }
     ```

  2. 在`ready_tasks`中删除它。

     ```c++
     ready_tasks.remove(shortest_task);
     ```

#### 3. 最短剩余时间优先`shortest_remaining_time_first`

从这一方案起，全都属于抢占式。任务可能分多次运行，`ready_tasks`要从`list<Task>`改为`list<TaskRuntime>`。

仍旧采用`first_future_task`加`ready_tasks`的模式，只是要同时维护`duration_left`。

```mermaid
flowchart TB
subgraph init[初始化]
    direction TB
    new_schedule[新建空 schedule]
    new_ready_tasks[新建空 ready_tasks]
    init_first_future_task["first_future_task = tasks.begin()"]
    zero[int clock = 0]
end

init --> while --> return[返回 schedule]

subgraph while["一直循环，直到没有未到达或就绪的任务"]
    handle_tasks_arrival
    --> select[从 ready_tasks 中<br>挑出 duration_left 最短的]:::diff
    --> 计算运行时间:::diff
    --> 记录并让时间流逝:::diff
    --> check{duration_left > 0}:::diff
    --> |是| back[压回 ready_tasks]:::diff
    
    check -->|否| 继续
end

classDef diff fill: orange;
```

- **`handle_tasks_arrival`**

  相当于原来的“必要时调表快进”和“更新新到达任务的状态”，只是整理成了函数。

- **计算运行时间**

  由于可抢占，任务不一定依次运行完，可能中途被强占，导致这次运行时间小于任务本身剩余运行时间。

  ```c++
  int duration = shortest_task.duration_left;
  if (first_future_task != tasks.end()) {
      duration = min(duration, first_future_task->arrive_at - clock);
  }
  ```

- **记录并让时间流逝**

  - 记录

    - 若上次记录（若有）和这次是同一任务，直接修改上次记录。
    - 否则在`schedule`末尾追加记录。

    这种奇怪操作在五种方案中独一无二。如果一个任务运行中途另一任务到达，需比较才知是否能抢占，即“调度”了一下。然而根据所给测试用例，这不算调度，前后两段要输出成一段。

  - 让时间流逝

    除了`clock += duration`，还要更新任务的剩余运行时间：`shortest_task.duration_left -= duration`。

- **压回`ready_tasks`**

  要压到`ready_tasks`末尾，这样保证“从`ready_tasks`中挑出`duration_left`最短的”时，若剩余时间相同，能挑出先来的。

#### 4. 时间片轮转`round_robin`

仍旧采用之前模式。

```mermaid
flowchart TB
subgraph init[初始化]
    direction TB
    new_schedule[新建空 schedule]
    new_ready_tasks[新建空 ready_tasks]
    init_first_future_task["first_future_task = tasks.begin()"]
    zero[int clock = 0]
end

init --> handle_tasks_arrival_0[handle_tasks_arrival]:::diff --> while --> return[返回 schedule]

subgraph while["一直循环，直到没有未到达或就绪的任务"]
    select[从 ready_tasks 中<br>挑出开头的]:::diff
    --> 计算运行时间:::diff
    --> 记录并让时间流逝
    --> handle_tasks_arrival:::diff
    --> check{duration_left > 0}
    --> |是| back[压回 ready_tasks]
    
    check -->|否| 继续
end

classDef diff fill: orange;
```

- **`handle_tasks_arrival`**

  内容与前面相同，只是位置变了。原来是在<u>上一轮</u>的“压回`ready_tasks`”<u>之后</u>，现在是在<u>这一轮</u>的“压回`ready_tasks`”<u>之前</u>。

  这是为了满足实验要求：任务回到`ready_tasks`时，要排到它运行时新到达的任务之。

- **从`ready_tasks`中挑出开头的**

  无需花 $\order{n}$ 找，第一个便是要运行的。

- **计算运行时间**

  ```c++
  int duration = min(next_task.duration_left, next_task.quantum);
  ```

#### 5. 动态优先级`dynamic_priority`

实验要求这里描述得较模糊，先重新阐释一下。

- 每次调度时选择运行的顺序：小优先数 → 早到达 → 小进程号。

  由于我`read_input`时已经按“早到达 → 小进程号”排好，遇到相同优先数，直接取靠前的即可。

- 指定初始优先数。

- 任务每运行一个时间片，优先数加3。

- 任务每在就绪队列中每停留一个时间片，优先数减1。

  不完整的时间片也算一个。换句话说，在时间片中途到达的任务，初次调度时优先级可能就已经变了。

- 优先数最小是零。

现在来看设计。仍旧采用`first_future_task`加`ready_tasks`的模式。下面是具体流程。

```mermaid
flowchart TB
subgraph init[初始化]
    direction TB
    new_schedule[新建空 schedule]
    new_ready_tasks[新建空 ready_tasks]
    init_first_future_task["first_future_task = tasks.begin()"]
    zero[int clock = 0]
end

init --> while --> return[返回 schedule]

subgraph while["一直循环，直到没有未到达或就绪的任务"]
    处理新到达任务:::diff
    --> select[在 ready_tasks 中<br>选择 priority 最小的]:::diff
    --> 运行
    
    subgraph 运行
        更新优先级:::diff
        --> 计算运行时间
        --> 记录并让时间流逝
	end

    运行
    --> check{duration_left == 0}:::diff
    --> |是| back[从 ready_tasks 中删除]:::diff
    
    check -->|否| 继续
end

classDef diff fill: orange;
```

- **处理新到达的任务**

  与原来的`handle_tasks_arrival`相同，但对于那些中途到达的任务，要先更新一次优先级。

  ```c++
  if (first_future_task->arrive_at < clock) {
      new_task.priority = max(new_task.priority - 1, 0);
  }
  ```

  注意这个`if`条件是严格不等号；而外层`while`条件和原来一样，仍是不严格的。

- **在`ready_tasks`中选择`priority`最小的**

  从“挑出”变成了“选择”。具体来说，运行任务前不再删除任务等后续加回，而是直接在原地操作，必要再删除。

  这样可以保持`ready_tasks`一直按“早到达 → 小进程号”排好，方便查找要运行的程序。

- **更新优先级**

  由于实验要求`schedule`中优先级是更新过的，所以要在“运行”开头更新。

  ```c++
  next_task->priority += 3;
  for (auto &&t : ready_tasks) {
      if (t != *next_task) {
          t.priority = max(t.priority - 1, 0);
      }
  }
  ```

## 实验结果及数据分析

### 乐学

全都通过了。

![](doc_1.assets/accepted.jpg)

![](doc_1.assets/截图 2022-09-20 21.21.26.png)

### 自己测试

测试了好多，这里只放调试最久的动态优先级的一个例子。

下图中，横坐标表示时刻，纵坐标对应任务编号；菱形表示到达（输入），矩形表示运行（输出）；菱形、矩形上的数字是当时的优先数；时间片为 40 ms（输入）。

```mermaid
gantt
dateFormat SSS
axisFormat %L ms

section 1
29: milestone, 200, 0
30: 270, 310
30: 430, 470
31: crit, 550, 570

section 2
26: milestone, 100, 0
28: 120, 160
30: 190, 230
31: 310, 350

section 3
27: milestone, 150, 0
29: 160, 190

section 4
30: milestone, 000, 0
33: crit, 000, 040
31: crit, 230, 270
31: 390, 430
32: 510, 550
33: 580, 620

section 5
28: milestone, 040, 0
31: crit, 040, 080
34: 080, 120
31: 350, 390
32: 470, 510
33: 570, 580

```

- 0 ms：只有任务 #4 到达了，故把 CPU 调度给他——未到达的任务不影响调度。
- 40 ms：此时任务 #4 的优先数为 33，#5为 28，调度给小的，即 #5。
- 230 ms：此时 #4 和 #1 的优先数都是 28，但 #4 比 #1 早到，所以调度给 #4。
- 570 ms：#1 只运行了 20 ms，因为用不完一整个时间片（40 ms）就完成了。

## 实验设计方法 2

### 重新分析

由于这里所有进程都仅有一段 CPU 计算，不会出现中断，可忽略“等待”，从而简化进程状态。

如下图，这里只采用`ready`、`running`两种状态，能触发三种事件引起调度，调度时会把某个进程从`ready`转为`running`。

```mermaid
stateDiagram-v2
direction LR

[*] --> ready: arrive
ready --> running: 调度
running --> ready: interrupt
running --> [*]: complete
```

相应建立`enum EventType`，包括`Arrive`、`Interrutpe`、`Complete`三种事件，再加上`PrivateUse`供某些特殊场景。

另外，这里的事件不可取消（一旦加入`events`，就不允许再删除）。

### 总体设计

大致思想是用类的继承、虚函数去除重复代码，同时使用事件循环替代初版各种`while`。

沿用`Algorithm`、`Task`、`TaskRuntime`，`SchedulerRecord`更名为`Record`，`Schedule`更名为`Plan`（以与下面的`Scheduler`区分）。尽量用迭代器替代引用，`list<TaskRuntime>::iterator`简称`TaskRuntimeIterator`。

首先规定`struct Event`，如下。其中`task_id`只适用于`Arrive`事件，其它时候都是`NOT_APPLICABLE`（`#define`的常量）。

```mermaid
classDiagram-v2

class Event {
    EventType type
    int at
    int task_id
}
```

接着是那些调度器类。

```mermaid
classDiagram-v2
direction LR

class Scheduler {
    +run() Plan

	#const list~Task~ &tasks
	#list~TaskRuntime~ working_tasks
	#TaskRuntimeIterator running_task
	#list~Event~ events

    #get_task(int id) TaskRuntime
    #register_event(Event event)*
    #register_arrivals()*
    #nothing_running() bool
    #handle_event(Event event, Plan &plan)*
    #on_arrive(Event event, Plan &plan)*
    #on_complete(Event event, Plan &plan)*
    #on_interrupt(Event event, Plan &plan)*
}

class SchedulerPreemptive {
	#on_interrupt(Event event, Plan &plan)*
	#handle_last_running_task()*
	#record_running_task(Plan &plan, int start_at, int end_at)*
	#next_task_to_run()* TaskRuntimeIterator
	#can_run_for(int now)* int
}

SchedulerPreemptive --|> Scheduler
```

> - 所有任务`tasks`。
> - 就绪和运行的任务`working_tasks`：默认还按“早到达 → 小进程号”排好，除非有特殊需要。
> - 未来的事件`events`：始终按发生顺序排列。
> - `get_task()`：从`tasks`中获取任务并转换为`TaskRuntime`。

```mermaid
classDiagram-v2
SchedulerPreemptive --|> Scheduler

class Scheduler {
    ……
}

class SchedulerPreemptive {
    ……
}

SchedulerFCFS --|> Scheduler
SchedulerSJF --|> Scheduler
SchedulerShortestRemainingTimeFirst --|> SchedulerPreemptive
SchedulerRoundRobin --|> SchedulerPreemptive
SchedulerDynamicPriority --|> SchedulerPreemptive

class SchedulerSJF {
    #on_arrive(Event event, Plan &plan)
}

class SchedulerShortestRemainingTimeFirst {
    #next_task_to_run() TaskRuntimeIterator
    #can_run_for(int now) int
    #record_running_task(Plan &plan, int start_at, int end_at)
}

class SchedulerRoundRobin {
    #can_run_for(int now) int
    #handle_last_running_task()
}

class SchedulerDynamicPriority {
    #next_task_to_run() TaskRuntimeIterator
    #can_run_for(int now) int
    #record_running_task(Plan &plan, int start_at, int end_at)
    #register_event(Event event)
    #handle_event(Event event, Plan &plan)
}

```

### 调度方案设计

## 实验结果及数据分析 2

> 乐学太慢了，直接在本地用`judger`和初版程序比较的。

### 自己测试

全都通过了。

![](doc_1.assets/image-20220920235753845.png)

## 总结

- 调试工具很重要。

  一开始 Wrong Answer 时就硬看，实在受不了……然后用 VS Code 的比较功能（如下图），好一点~儿~，但还是不容易搞清关系。最后做了`draw_gantt.py`，很舒适。

  ![](doc_1.assets/image-20220920222726869.png)

- 区分传值和传引用。

  `ready_tasks.front()`虽然返回`TaskRuntime &`，但`auto t = ready_tasks.front()`（而非`auto &`）的`t`仍然是`TaskRuntime`。因而修改`t.priority`不会影响`ready_tasks`里的优先级。

  我这次最后采用`auto t = ready_tasks.begin()`，`t`是`list<TaskRuntime>::iterator`，肯定能修改原链表里的优先级。这样还可用`erase`替代`remove`，从而避免大量比较。（不过没有太大影响）

- 代码本身就暗示了极端情况。

  例如一写下`auto shortest_task = ready_tasks.front()`，就要想想`ready_tasks`空不空。

## 附录

### 程序清单及说明

- `draw_gantt.py`：绘制 Gantt 图。
- `ex_1-event.cpp`：去除重复代码的版本。
- `ex_1.cpp`：初次写的版本。
- `ref.py`：从[网上找的（部分）正确程序](https://www.jianshu.com/p/6e415fdce561)的节选，用于反推实验要求。
- `/judger/src/`
  - `lib.rs`：解析参数、用测试用例检查程序。
  - `main.rs`：CLI。
  - `run.rs`：较底层的运行。

### `draw_gantt.py`使用示例

```powershell
> ./draw_gantt.py
algorithm id >> 5
tasks >>
1/200/100/29/40
……
5/40/170/28/40
^Z
schedule >>
1/4/0/40/33
……
17/4/580/620/33
^Z
gantt
dateFormat SSS
axisFormat %L ms

section 1
29: milestone, 200, 0
30: 270, 310

……

section 5
……
33: 570, 580

I've copied to your clipboard. Try to paste it to https://mermaid.live/ .
```

> `^Z`是<kbd>Ctrl</kbd>+<kbd>Z</kbd>。
