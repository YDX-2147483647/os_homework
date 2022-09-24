# 实验2 读者写者问题

## 实验内容

在Windows环境下，创建一个控制台进程，此进程包含n个线程。用这n个线程来表示n个读者或写者。每个线程按输入的要求进行读写操作。用信号量机制分别实现读者优先和写者优先问题。

运行结果显示要求：要求在每个线程创建、发出读写申请、开始读写操作和结束读写操作时分别显示一行提示信息，以确定所有处理都遵守相应的读写操作限制。

## 实验目的

1. 通过编写和调试程序以加深对进程、线程管理方案的理解。
2. 熟悉Windows多线程程序设计方法。

## 实验基础知识

> 以下很多地方的“线程”换成“进程”同样适用，甚至更合理。不过这次实验是多线程，所以都写的是“线程”。

### 同步问题

多个线程可能希望协作使用同一资源，需要尽可能实现互斥使用、有空让进、优先等待。

### 信号量

信号量（semaphore）用于解决同步问题，代表可用的资源。

具体来说，信号量包括一个整数，涉及如下两种原子操作。

```rust
/// 等待空闲资源并获取
fn wait(semaphore) {
    semaphore.value -= 1;
    if semaphore.value < 0 {
        // 若无可用资源，等待别人用完通知
        semaphore.list.push(this_process);
        sleep();
    }
}

/// 释放资源并通知他人可以利用
fn signal(semaphore) {
    semaphore += 1;
    if semaphore.value <= 0 {
        let process = semaphore.list.pop();
        wake_up(process);
    }
}
```

在多个线程各自`wait`、`signal`同一公用信号量，可以解决同步问题。在每一线程先后`wait`、`signal`同一私用信号量，可以解决互斥问题。

### 读者—写者问题

读者—写者问题读写操作的通用限制：

- 写—写互斥：不能有两个写者同时进行写操作。

- 读—写互斥：不能同时有一个线程在读，而另一个线程在写。

- 读—读允许：可以有一个或多个读者在读。

附加限制：

- **读者优先**：读者申请时，只要已有其它读者正在读，则它可直接开始操作，不理会写者的请求。
- **写者优先**：一旦有写者申请，任何新读者都必须先等待。
- **公平竞争**：所有操作者都要在`service`的等待队列中排队，从而保证公平。

## 实验设计方法

> 另请参阅自动生成的文档：`cargo doc --open`或手动打开`target/doc/ex_2/index.html`。那里会有更细节的东西，比如`id`存储时是用多少位整数。

### 操作员`Operator`（`operator.rs`）

#### 结构

针对输入设计`Operator`操作员。（如下

```mermaid
classDiagram-v2
class Operator {
    +id: u32 序号
    +role: OperatorRole 角色
    +start_at: f32 操作开始时刻，单位为秒，正数
    +duration: f32 操作持续时间，正数
}
```

其中角色`role: OperatorRole`如下。

```rust
#[derive(Debug, PartialEq)]
pub enum OperatorRole {
    Reader,
    Writer,
}
```

#### 功能

- `Operator::from(line: &str) -> Result<Operator, OperatorParseError>`

  将字符串（如`"1 R 3.3 5"`）解析为`Operator`。

  > 解析字符串要干很多琐碎的事，抛出各种错误（`enum OperatorParseError`）。然而没太大关系，就不多介绍了。

- `ready_inputs() -> Vec<Operator>`

  从 stdin 读取若干行，解析为`Operator`列表。

### 同步方案`run_○○(…)`（`solutions.rs`）

每种同步方案写成一个函数。

- **输入**：操作员列表（`operators: Vec<Operator>`）。

- **功能**：

  1. 建立同步手段（信号量等）。

  2. 让每位操作员在一个线程运行。

     （运行时并不真的读写文件，只是用`thread::sleep`模拟）

  3. 等待所有操作员结束。

- **没有的功能**：打印运行记录。

  `run_○○()`会向外发送消息（`tx.send((…))`）来传递运行记录，把这些记录打印出来是`Reporter`的工作。

  > 实际上`run_○○()`还需要另一参数接入管道——`tx: Sender<ReportMessage>`。这些东西会在`Reporter`一节介绍。

#### 读者优先方案`run_read_preferring`

读者申请时，只要已有其它读者正在读，则它可直接开始操作，不理会写者的请求。

##### 原理

1. 由于互斥访问（读者团体—写者、写者—另一写者），需用**信号量`access`**表示文件的读写权。

   谁占有`access`（把`acess`改成假），谁就能操作文件。

   初始时，所有人都可随时夺权，因此取真。

   ```rust
   let access = Arc::new((Mutex::new(true), Condvar::new()));
   ```

   > 关于信号量为什么写成这样，会在`Semaphore`一节介绍。

2. 读者作为一个团体和其他人互斥，团体内部可允许同时访问。因此，团体中第一人夺权，最后一人放权。

   由于“第一人”和“最后一人”往往不是同一人，必须操作员间通信才能实现，故设置**计数器`n_readers`**记录当前**正在访问的读者**数量。

   初始时，无读者在访问，取零。

   ```rust
   let mut n_readers = 0; // 部分正确
   ```

3. 读者整个团体要共同维护`n_readers`一个变量，必须互斥访问。

   这里使用**互斥锁`Mutex`**实现。上一行修改如下。

   ```rust
   let n_readers = Arc::new(Mutex::new(0));
   ```

4. 现在**检查**一下读者是否真的优先。

   读者申请时，只要已有其它读者正在读，就说明读者团体已经把`access`夺了，则它可直接开始操作，不理会写者的请求。

   而写者申请时，夺到`access`前必须等整个读者团体放权，即所有读者都读完了。

##### 实现

- **总体**

  ```mermaid
  flowchart TB
  subgraph 初始化
      direction TB
      init_access["access = Semaphore(true)"]
      init_n["n_readers = Mutex(0)"]
      %% init_now["now = Instant::now()"]
  end
  
  初始化 --> for
  
  subgraph for["for o in operators"]
      match[o.role]
      -->|Reader| spawn_r[新建读者线程]
      match -->|Writer| spawn_w[新建写者线程]
  end
  ```

  > 因为操作员会向主线程的`Reporter`发送运行记录，主线程天然会等待这些线程，无需`join`。

  大架子都是这一套，后面就不重复了。

- **写者**

  ```mermaid
  flowchart LR
  create --> sleep["sleep(o.start_at)"]
  --> wait["wait(access)"]:::sema
  --> write:::crit
  --> signal["signal(access)"]:::sema
  
  subgraph access
      write
      signal
  end
  
  classDef crit fill: red;
  classDef sema fill: orange;
  ```

  > 这一节会展示流程图怎么对应到代码，之后就不展示了。

  ```rust
  thread::spawn(move || {
      tx.send((o.id, Action::Create, now.elapsed())).unwrap();
      // ↑ 向外发送运行记录，后同。
  
      thread::sleep(Duration::from_secs_f32(o.start_at));
  
      tx.send((o.id, Action::RequestWrite, now.elapsed()))
          .unwrap();
      wait(&*access);
  
      tx.send((o.id, Action::StartWrite, now.elapsed())).unwrap();
      thread::sleep(Duration::from_secs_f32(o.duration));
      tx.send((o.id, Action::EndWrite, now.elapsed())).unwrap();
  
      signal(&*access);
  })
  ```

- **读者**

  ```mermaid
  flowchart TB
  create --> sleep["sleep(o.start_at)"]
  --> enter
  --> write:::crit
  --> exit
  
  subgraph enter["n_readers的互斥锁"]
      increase["*n_readers += 1"]
      --> if_first{*n_readers == 1}
      -->|是| wait["wait(access)"]:::sema
  end
  
  subgraph exit["n_readers的互斥锁"]
      decrease["*n_readers -= 1"]
      --> if_last{*n_readers == 0}
      -->|是| signal["signal(access)"]:::sema
  end
  
  classDef crit fill: red;
  classDef sema fill: orange;
  ```

  ```rust
  thread::spawn(move || {
      tx.send((o.id, Action::Create, now.elapsed())).unwrap();
  
      thread::sleep(Duration::from_secs_f32(o.start_at));
  
      tx.send((o.id, Action::RequestRead, now.elapsed())).unwrap();
      {
          let mut n_readers = n_readers.lock().unwrap();
          *n_readers += 1;
  
          // if I am the first
          if *n_readers == 1 {
              wait(&*access);
          }
      }
  
      tx.send((o.id, Action::StartRead, now.elapsed())).unwrap();
      thread::sleep(Duration::from_secs_f32(o.duration));
      tx.send((o.id, Action::EndRead, now.elapsed())).unwrap();
  
      {
          let mut n_readers = n_readers.lock().unwrap();
          *n_readers -= 1;
  
          // if I am the last
          if *n_readers == 0 {
              signal(&*access);
          }
      }
  })
  ```

#### 写者优先方案`run_write_preferring`

一旦有写者申请，任何新读者都必须先等待。

##### 原理

1. 同前，设计信号量`access`、计数器`n_readers`、`n_readers`的互斥锁。

2. 新读者有时要因写者而等待，这涉及通信，肯定要另设计**信号量`can_reader_acquire`**。

   > 在我的程序中，request 对应整个申请权限的过程，acquire 表示申请`access`。如果对外封装，那么看得到 request，看不到 acquire。

   初始时，无写者，读者总可申请，因此取真。

   ```rust
   let can_reader_acquire = Arc::new((Mutex::new(true), Condvar::new()));
   ```

3. 存在写者等待时，新读者要延后申请`access`，否则无需。“存在等待写者与否”说明需设置**计数器`n_writers`**记录当前**正在等待或访问的写者**数量。

   注意`n_writers`也算那些正在等`access`的写者，而`n_readers`不计。事实上由于读者团体内部不互斥，他们根本不存在“等`access`”这一状态。

   初始时无写者，为零。

4. 同理，`n_writers`也需要**互斥锁**。

   ```rust
   let n_writers = Arc::new(Mutex::new(0));
   ```

5. 怎样将“存在等待写者与否”转换为`can_reader_acquire`？有写者等待或访问时，把`can_reader_acquire`抢走（副作用：改为`false`）。也就是说，第一位写者来时`wait`（阻塞所有新读者），最后一位走时`signal`（通知新读者可以申请`access`）。

##### 实现

- **总体**

  ```mermaid
  flowchart LR
  subgraph 初始化
      direction LR
      init_access["access = Semaphore(true)"]
      init_n["n_readers = Mutex(0)"]
      n_w["n_writers = Mutex(0)"]
      can["can_reader_acquire = Semaphore(true)"]
  end
  
  初始化 --> ell["…"]
  ```

- **写者**

  ```mermaid
  flowchart TB
  create --> sleep["sleep(o.start_at)"]
  --> enter
  --> wait["wait(access)"]:::sema
  --> write:::crit
  --> signal["signal(access)"]:::sema
  --> exit
  
  subgraph access
      write
      signal
  end
  
  subgraph enter["n_writers的互斥锁"]
      increase["*n_writers += 1"]
      --> if_first{*n_writers == 1}
      -->|是| wait_can["wait(can_reader_acquire)"]:::sema
  end
  
  subgraph exit["n_writers的互斥锁"]
      decrease["*n_writers -= 1"]
      --> if_last{*n_writers == 0}
      -->|是| signal_can["signal(can_reader_acquire)"]:::sema
  end
  
  classDef crit fill: red;
  classDef sema fill: orange;
  ```

- **读者**

  ```mermaid
  flowchart TB
  create --> sleep["sleep(o.start_at)"]
  --> wait_can["wait(can_reader_acquire)"]:::sema
  --> enter
  --> signal_can["signal(can_reader_acquire)"]:::sema
  --> write:::crit
  --> exit
  
  subgraph enter["n_readers的互斥锁"]
      increase["*n_readers += 1"]
      --> if_first{*n_readers == 1}
      -->|是| wait["wait(access)"]:::sema
  end
  
  subgraph can_reader_acquire
      enter
      signal_can
  end
  
  subgraph exit["n_readers的互斥锁"]
      decrease["*n_readers -= 1"]
      --> if_last{*n_readers == 0}
      -->|是| signal["signal(access)"]:::sema
  end
  
  classDef crit fill: red;
  classDef sema fill: orange;
  ```

#### 公平竞争`run_unspecified_priority`

所有操作员都要一起排队，从而保证公平。

##### 原理

1. 同前，设计信号量`access`、计数器`n_readers`、`n_readers`的互斥锁。

2. 所有操作员排的队是一个信号量的等待队列，这个**信号量**称作**`service`**。

   初始时，队是空的，取真即可。

   ```rust
   let service = Arc::new((Mutex::new(true), Condvar::new()));
   ```

   所有操作员申请`access`时都要在`service`排队，申请前`wait`，申请后`signal`。

3. **检查**一下有没有破坏读者团体内允许。

   读者的`service`区间几乎和`n_readers`的互斥锁一致，所以没破坏。

##### 实现

- **总体**

  ```mermaid
  flowchart LR
  subgraph 初始化
      direction LR
      init_access["access = Semaphore(true)"]
      init_n["n_readers = Mutex(0)"]
      service["service = Semaphore(true)"]
  end
  
  初始化 --> ell["…"]
  ```

- **写者**

  ```mermaid
  flowchart LR
    create --> sleep["sleep(o.start_at)"]
    --> wait_service["wait(service)"]:::sema
    --> wait["wait(access)"]:::sema
    --> signal_service["signal(service)"]:::sema
    --> write:::crit
    --> signal["signal(access)"]:::sema
    
    subgraph service
        wait
        signal_service
    end
    
    subgraph access
        write
        signal
    end
    
    classDef crit fill: red;
    classDef sema fill: orange;
  ```

- **读者**

  ```mermaid
  flowchart LR
  create --> sleep["sleep(o.start_at)"]
  --> wait_service["wait(service)"]:::sema
  --> enter
  --> signal_service["signal(service)"]:::sema
  --> write:::crit
  --> exit
  
  subgraph service
      enter
      signal_service
  end
  
  subgraph enter["n_readers的互斥锁"]
    increase["*n_readers += 1"]
    --> if_first{*n_readers == 1}
    -->|是| wait["wait(access)"]:::sema
  end
  
  subgraph exit["n_readers的互斥锁"]
    decrease["*n_readers -= 1"]
    --> if_last{*n_readers == 0}
    -->|是| signal["signal(access)"]:::sema
  end
  
  classDef crit fill: red;
  classDef sema fill: orange;
  ```

### 信号量`Semaphore`（`semaphore.rs`）

#### 背景

因为种种原因，Rust 标准库中的`sync::Semaphore`已经被淘汰了。在共享内存范畴内，可采用以下工具。

- **互斥锁**`sync::Mutex`（mutual exclusion）

  类似于只取两个值的信号量。

  ```rust
  let data = Arc::new(Mutex::new(0));
  
  thread::spawn(move || {
      // --snip--
      {
          let mut data = data.lock().unwrap();
          *data += 1;
      }
      // --snip--
  });
  ```

  `data.lock()`拿锁，拿到前一直阻塞；它结束生命时释放锁。——互斥锁解决互斥问题，和私用信号量一样，获取锁和释放锁是在同一线程。

  > 如果某条线程拿着锁时炸了，其它线程试图拿锁时`data.lock()`会返回`None`。

- **条件变量**`sync::Condvar`（condition variable）

  条件变量传递一个逻辑变量，可以阻塞线程。

  ```rust
  // --snip--
  
  // 子线程修改后通知主线程
  thread::spawn(move|| {
      let (lock, cvar) = &*pair2;
      let mut started = lock.lock().unwrap();
      *started = true;
      cvar.notify_one();
  });
  
  // 主线程在收到通知前一直阻塞
  let (lock, cvar) = &*pair;
  let mut started = lock.lock().unwrap();
  while !*started {
      // 这里不会忙等待，因为大部分时间阻塞在下面这行
      started = cvar.wait(started).unwrap();
  }
  ```

#### 设计

- 私用信号量（互斥问题）：直接用`Mutex`。
- 公用信号量（同步问题）：使用自制信号量`Semaphore`。

最后我发现自制信号量非常简单……

我们把`Mutex`、`Condvar`对~儿~当作信号量。前者保证原子性，后者阻塞线程。

```rust
use std::sync::{Condvar, Mutex};

type Semaphore = (Mutex<bool>, Condvar);
```

> 因为本实验用到信号量的地方都只有一个资源，我就直接把信号量的值设计成`bool`了。

下面来看 P、V 操作。

```rust
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
```

- 二者都是原子操作，上来都先用`lock`锁住。

  > 随即用信号量的值覆盖原来的`lock`变量。

- `wait`

  - 若有剩余资源，`*lock == true`，`while`进不去，直接占有资源（`*lock = false`），返回。
  - 否则，用`cvar`阻塞当前线程。直到有人释放资源，然后重试。

- `signal`

  释放资源（`*lock = true`），用`cvar`通知他人。

### `Reporter`

- 共享状态
- 消息传递

## 实验结果及数据分析

### 读者优先

```powershell
> cat .\test_cases\mixed.in | cargo run -- read-preferring --tab 10
 0.000 s |          #1：🚀创建。
 0.000 s |                                        #4：🚀创建。
 0.000 s |                              #3：🚀创建。
 0.000 s |                    #2：🚀创建。
 0.000 s |                                                  #5：🚀创建。
 3.013 s |          #1：🔔👀申请读取。
 3.013 s |          #1：🏁👀开始读取。
 4.008 s |                    #2：🔔📝申请写入。
 5.004 s |                              #3：🔔👀申请读取。
 5.004 s |                              #3：🏁👀开始读取。
 5.103 s |                                                  #5：🔔📝申请写入。
 6.007 s |                                        #4：🔔👀申请读取。
 6.007 s |                                        #4：🏁👀开始读取。
 7.012 s |                              #3：🛑👀结束读取。
 8.017 s |          #1：🛑👀结束读取。
11.016 s |                                        #4：🛑👀结束读取。
11.016 s |                    #2：🏁📝开始写入。
16.028 s |                    #2：🛑📝结束写入。
16.028 s |                                                  #5：🏁📝开始写入。
19.033 s |                                                  #5：🛑📝结束写入。
```

```mermaid
gantt
dateFormat ss.SSS
axisFormat %S.%L s

section 1
🚀: milestone, 00.000, 0
🔔👀: milestone, 03.013, 0
👀: 03.013, 08.017

section 2
🚀: milestone, 00.000, 0
🔔📝: milestone, 04.008, 0
📝: 11.016, 16.028

section 3
🚀: milestone, 00.000, 0
🔔👀: milestone, 05.004, 0
👀: 05.004, 07.012

section 4
🚀: milestone, 00.000, 0
🔔👀: milestone, 06.007, 0
👀: 06.007, 11.016

section 5
🚀: milestone, 00.000, 0
🔔📝: milestone, 05.103, 0
📝: 16.028, 19.033
```

### 写者优先

```powershell
> cat .\test_cases\mixed.in | cargo run -- write-preferring --tab 10
 0.000 s |          #1：🚀创建。
 0.000 s |                              #3：🚀创建。
 0.000 s |                    #2：🚀创建。
 0.000 s |                                        #4：🚀创建。
 0.000 s |                                                  #5：🚀创建。
 3.009 s |          #1：🔔👀申请读取。
 3.009 s |          #1：🏁👀开始读取。
 4.010 s |                    #2：🔔📝申请写入。
 5.003 s |                              #3：🔔👀申请读取。
 5.101 s |                                                  #5：🔔📝申请写入。
 6.012 s |                                        #4：🔔👀申请读取。
 8.022 s |          #1：🛑👀结束读取。
 8.022 s |                    #2：🏁📝开始写入。
13.026 s |                    #2：🛑📝结束写入。
13.026 s |                                                  #5：🏁📝开始写入。
16.031 s |                                                  #5：🛑📝结束写入。
16.031 s |                              #3：🏁👀开始读取。
16.031 s |                                        #4：🏁👀开始读取。
18.041 s |                              #3：🛑👀结束读取。
21.033 s |                                        #4：🛑👀结束读取。
```

```mermaid
gantt
dateFormat ss.SSS
axisFormat %S.%L s

section 1
🚀: milestone, 00.000, 0
🔔👀: milestone, 03.009, 0
👀: 03.009, 08.022

section 2
🚀: milestone, 00.000, 0
🔔📝: milestone, 04.010, 0
📝: 08.022, 13.026

section 3
🚀: milestone, 00.000, 0
🔔👀: milestone, 05.003, 0
👀: 16.031, 18.041

section 4
🚀: milestone, 00.000, 0
🔔👀: milestone, 06.012, 0
👀: 16.031, 21.033

section 5
🚀: milestone, 00.000, 0
🔔📝: milestone, 05.101, 0
📝: 13.026, 16.031
```

### 公平竞争

```powershell
> cat .\test_cases\mixed.in | cargo run -- unspecified-priority --tab 10
 0.000 s |          #1：🚀创建。
 0.000 s |                    #2：🚀创建。
 0.000 s |                              #3：🚀创建。
 0.000 s |                                        #4：🚀创建。
 0.000 s |                                                  #5：🚀创建。
 3.009 s |          #1：🔔👀申请读取。
 3.009 s |          #1：🏁👀开始读取。
 4.007 s |                    #2：🔔📝申请写入。
 5.001 s |                              #3：🔔👀申请读取。
 5.104 s |                                                  #5：🔔📝申请写入。
 6.019 s |                                        #4：🔔👀申请读取。
 8.019 s |          #1：🛑👀结束读取。
 8.019 s |                    #2：🏁📝开始写入。
13.044 s |                    #2：🛑📝结束写入。
13.044 s |                              #3：🏁👀开始读取。
15.050 s |                              #3：🛑👀结束读取。
15.050 s |                                                  #5：🏁📝开始写入。
18.054 s |                                                  #5：🛑📝结束写入。
18.054 s |                                        #4：🏁👀开始读取。
23.077 s |                                        #4：🛑👀结束读取。
```

```mermaid
gantt
dateFormat ss.SSS
axisFormat %S.%L s

section 1
🚀: milestone, 00.000, 0
🔔👀: milestone, 03.009, 0
👀: 03.009, 08.019

section 2
🚀: milestone, 00.000, 0
🔔📝: milestone, 04.007, 0
📝: 08.019, 13.044

section 3
🚀: milestone, 00.000, 0
🔔👀: milestone, 05.001, 0
👀: 13.044, 15.050

section 4
🚀: milestone, 00.000, 0
🔔👀: milestone, 06.019, 0
👀: 18.054, 23.077

section 5
🚀: milestone, 00.000, 0
🔔📝: milestone, 05.104, 0
📝: 15.050, 18.054
```

## 总结

## 附录

程序清单及说明。（列出文件名及说明即可，不需要在此处复制代码，代码直接以源文件形式提供，但源文件中对代码要有必要的注释和说明）
