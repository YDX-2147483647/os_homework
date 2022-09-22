# 读者写者问题

```powershell
> cat ./test_cases/mixed.in | cargo run
# mixed.in 是提供的测试输入
> cat ./test_cases/gap.in | cargo run
```

## 相关链接

- The Rust Book, [Shared state concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html).
- Stack Overflow, [Deprecation of `std::sync::Semaphore` and its reason](https://stackoverflow.com/questions/59480070/replacement-for-stdsyncsemaphore-since-it-is-deprecated).
- Docs.rs, [`tokio::sync::Semaphore`](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).
- Docs.rs, [`semaphore::Semaphore`](https://docs.rs/semaphore/latest/semaphore/struct.Semaphore.html).
