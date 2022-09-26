# 读者写者问题

## 使用说明和示例

### 查看帮助

```powershell
> cargo run -- --help
ex_2 0.1.0

USAGE:
    ex_2.exe [OPTIONS] <POLICY>

ARGS:
    <POLICY> ……
…………
```

若没有 [Cargo](https://doc.rust-lang.org/cargo/index.html)，可输入`/target/release/ex_2.exe --help`，即将`cargo run --`替换为`./target/release/ex_2.exe`，后同。

### 按“读者优先”策略处理`mixed.in`

```powershell
> cat ./test_cases/mixed.in | cargo run -- read-preferring
 0.000 s | #1：🚀创建。
…………
 3.013 s | #1：🔔👀申请读取。
 3.013 s | #1：🏁👀开始读取。
 4.000 s | #2：🔔📝申请写入。
…………
```

- 策略可以是读者优先（`read-preferring`）、写者优先（`write-preferring`）、公平竞争（`unspecified-priority`）。

- `mixed.in`是一个文本文件，内容是马老师或王老师提供的测试输入。

  ```powershell
  > cat .\test_cases\mixed.in
  1 R 3 5
  2 W 4 5
  3 R 5 2
  4 R 6 5
  5 W 5.1 3
  ```

  格式如下。

  > 测试数据文件包括 n 行测试数据，分别描述创建的 n 个线程是读者还是写者，以及读写操作的开始时间和持续时间。
  >
  > 每行测试数据包括四个字段，每个字段间用空格分隔。
  >
  > 1. 第1个字段为正整数，表示线程的序号。
  > 2. 第2个字段表示线程的角色，R表示读者，W表示写者。
  > 3. 第3个字段为一个正数，表示读写开始时间：线程创建后，延迟相应时间（单位为秒）后发出对共享资源的读写申请。
  > 4. 第4个字段为一个正数，表示读写操作的延迟时间。当线程读写申请成功后，开始对共享资源进行读写操作，该操作持续相应时间后结束，释放该资源。

- 输出运行记录。

  格式如下。

  ```
   4.000 s | #2：🔔📝申请写入。
  <自运行开始的时间> | #<操作员 id>：<动作>。
  ```

  动作有以下几种。

  - 🚀创建
  - 🔔申请……
  - 🏁开始……
  - 🛑结束……
  - 👀……读取
  - 📝……写入

也可直接`cargo run -- read-preferring`，然后一行一行输入`1 R 3 5`之类的，最后按<kbd>Ctrl</kbd>+<kbd>Z</kbd>结束输入。

### 打印信息时缩进

```powershell
> cat ./test_cases/gap.in | cargo run -- write-preferring --tab 10
 0.000 s |          #1：🚀创建。
 0.000 s |                    #2：🚀创建。
…………
 1.014 s |          #1：🔔👀申请读取。
 1.014 s |          #1：🏁👀开始读取。
 1.529 s |          #1：🛑👀结束读取。
 3.008 s |                    #2：🔔📝申请写入。
 3.009 s |                    #2：🏁📝开始写入。
 3.517 s |                    #2：🛑📝结束写入。
 …………
```

- `tab`选项的值控制打印信息时每个进程缩进的数量。

  这样每位操作员独占一列，看起来更简单，

  例如`--tab 10`表示操作员 #1 缩进 1×10 = 10 个空格，操作员 #2 缩进 20 个空格，以此类推。

- `gap.in`是另一个测试输入。

### Gantt 图

```powershell
> cat ./test_cases/mixed.in | cargo run -- read-preferring
…………
19.025 s | #5：🛑📝结束写入。
I've copied to your clipboard. Try to paste it into https://mermaid.live/ .
```

运行完后，你的剪贴板内容会类似下面这样。

```
gantt
dateFormat ss.SSS
axisFormat %S.%L s

section 1
🚀: milestone, 00.000, 0
🔔👀: milestone, 01.014, 0
👀: 01.014, 01.529

…………
```

访问 [Mermaid Live Editor (mermaid.live)](https://mermaid.live/)，粘贴到 Code 一栏，可在 Diagram 一栏画出 Gantt 图。

![](ReadMe.assets/mermaid_live.png)

## 开发相关的链接

- The Rust Book, [Shared state concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html).
- Stack Overflow, [Deprecation of `std::sync::Semaphore` and its reason](https://stackoverflow.com/questions/59480070/replacement-for-stdsyncsemaphore-since-it-is-deprecated).
- Docs.rs, [`tokio::sync::Semaphore`](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).
- Docs.rs, [`semaphore::Semaphore`](https://docs.rs/semaphore/latest/semaphore/struct.Semaphore.html).
