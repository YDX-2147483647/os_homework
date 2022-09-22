# [实验2 读者写者问题](https://lexue.bit.edu.cn/mod/assign/view.php?id=365474)

> 2022年9月18日。

并发操作——atomicity: all or none.

问题类型：

- 读者优先
- 写者优先
- 公平竞争／自定义

本质是互斥。

- 读者优先的解法
  - 建立读者计器
  - 建立合理的信号量（临界资源）
  - PV操作
- 独立思考
- 写者优先：
  - 如果有写者申请写文件，那么在申请之前已经开始读取文件的可以继续读取。
  - 但是如果再有读者申请读取文件，则不能够读取，只有在所有的写者写完之后才可以读取。
- 公平竞争？

> 线下签到是为了回溯密切接触者。

## 写者优先时不小心死锁了

> 2022年9月22日。

下面是`Writer`线程的一部分。

```rust
reporter.report(&o, Action::Request);
{
    let mut n_writers = n_writers.lock().unwrap();
    *n_writers += 1;

    // if I am the first
    if *n_writers == 1 {
        wait(&*can_reader_acquire);
    }
    wait(&*access); // ←
}

reporter.report(&o, Action::Start);
thread::sleep(Duration::from_secs_f32(o.duration));
reporter.report(&o, Action::End);

{
    let mut n_writers = n_writers.lock().unwrap();
    *n_writers -= 1;

    // if I am the last
    if *n_writers == 0 {
        signal(&*can_reader_acquire);
    }
    signal(&*access); // ←
}
```

```powershell
> cat .\test_cases\mixed.in | cargo run -- write-preferring --tab 10
 0.000 s |          #1：🚀创建。
 0.000 s |                    #2：🚀创建。
 0.000 s |                              #3：🚀创建。
 0.000 s |                                        #4：🚀创建。
 0.000 s |                                                  #5：🚀创建。
 3.004 s |          #1：🔔👀申请读取。
 3.005 s |          #1：🏁👀开始读取。
 4.021 s |                    #2：🔔📝申请写入。
 5.010 s |                              #3：🔔👀申请读取。
 5.110 s |                                                  #5：🔔📝申请写入。
 6.020 s |                                        #4：🔔👀申请读取。
 8.024 s |          #1：🛑👀结束读取。
 8.025 s |                    #2：🏁📝开始写入。
13.030 s |                    #2：🛑📝结束写入。
```

运行到这里会卡住，因为下一个写者（#5）已经拿着`n_writters`的锁在`wait(&*access)`，可现在的写者（#2）不拿到`n_writters`的锁就无法`signal(&*access)`。

> 我当时先试验出解决办法（尽量让`wait`、`signal`顺序相反），然后才反应过来怎么回事……

解决办法：

- 将`wait(&*access)`向后挪出`n_writers`的锁。
- 将`signal(&*access)`向前挪出`n_writers`的锁。

执行任意一种办法即可打破死锁，最后我两种都采取了。
