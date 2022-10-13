# 实验3 请求页式存储管理

## 实验内容

编写一个请求页式存储管理程序，模拟请求页式存储管理方式下的内存分配和页面置换。

## 实验目的

内存管理是操作系统中的核心模块，能够合理利用内存，在很大程度上将影响到整个计算机系统的性能。内存的分配和回收与内存管理方式有关。

本实验要求学生独立设计并实现请求页式存储管理方式下的内存分配与页面置换模拟程序，以加深对页面置换算法和请求页式存储管理方式的理解。

## 实验基础知识

- **存储管理**

  存储管理需要解决分配、重定位、保护、扩充和共享问题。

- **请求页式存储管理**

  把主存划分成一系列等大的物理页框，给每个进程分配若干整的物理页框，体现为逻辑页面。

  请求逻辑页面时，若内容尚未读入主存，会发生“缺页中断”，待读入后重新运行。

  读入时，若没有空物理页框可分配，需要用新的内容替换旧有页框，即“页面置换”。

  读入主存相对 CPU 运算很慢，页面置换算法最好尽量降低缺页次数。

## 实验设计方法

### 总体设计

#### 结构

首先针对输入输出设计结构。

> 以下如无说明，皆为`struct`。

逻辑页面号全部用`int`表示。

- **输入方面**

  - 页面置换策略`Policy`。（枚举）
  - 输入`Input`。

  ```mermaid
  classDiagram-v2
      class Input {
          Policy policy 页面置换策略
          unsigned int n_frames 物理页框数量
          vector~int~ pages 页面请求序列
      }
  ```

  ```c++
  enum Policy {
      Optimal = 1,
      FirstInFirstOut = 2,
      LeastRecentlyUsed = 3,
  };
  ```

- **输出方面**

  - “页表”`PageTable`（`vector<int>`的别名）。

    索引是物理页框号，内容是逻辑页面号，`IDLE`（`-1`）表示空闲。

    > 页表用于重定位，逻辑页面 → 物理页框；而这里的`PageTable`记录存储情况，是反过来映射（物理页框 → 逻辑页面），严格来说并非页表。（但我想不出更好的名字）

  - 页面更改`PageChange`。

  - 输出`vector<PageChange>`。

  ```mermaid
  classDiagram-v2
      class PageChange {
          PageTable table 更改后的“页表”
          bool hit 是否命中
      }
  ```

#### 功能和流程

```mermaid
flowchart LR
read_inputs -->|input| 匹配要求的策略 -->|manager| 模拟请求页面 -->|changes| write_outputs
```

1. **`read_inputs`**

   从`stdin`读入数据，验证合法性，解析为`Input`。

   ```mermaid
   flowchart
   创建[创建 input: Input]
   --> in_policy
   --> in_n_frames[输入 input.n_frames]
   --> in_pages
   --> return[返回 input]
   
   subgraph in_policy[输入 input.policy]
       输入整数 --> 断言["断言：编号合法"] --> set_policy["input.policy = Policy(policy)"]
   end
   
   subgraph in_pages[输入 input.pages]
      buffer[创建 buffer: string]
      --> getline["读取至下一逗号，写入 buffer"]
      -->|成功| stoi[解析 buffer 为整数]
      --> push["追加到 input.pages 末尾"]
      --> getline
      
      getline -->|失败| 结束循环
   end
   ```

   > 其中输入`input.pages`时，使用`getline(cin, buffer, ',')`实现。

2. **匹配要求的策略**

   用`switch`–`case`匹配，==实例化相应`Manager`子类==。

   如果输入非法，调用`not_implemented()`向`stderr`报错并结束程序。

   ```c++
   Manager *manager = nullptr;
   
   switch (input.policy) {
   case Policy::□□:
       manager = new Manager□□(input.n_frames);
       break;
   ……
   }
   ```

   > 这个`manager`指针会在`main()`结尾`delete`。

3. **模拟请求页面**

   `Manager`子类==各自实现了`request`方法==，这里调用即可。

   ```c++
   auto changes = manager->request(input.pages);
   ```

4. **`write_outputs(changes)`**

   遍历`changes`，按格式输出页面变化情况，==记录缺页次数==并打印。

   ```mermaid
   flowchart TB
   初始化计数器 --> for --> out_n[输出 n_page_faults]
   
   subgraph 初始化计数器
       direction TB
       init_n_page_faults[n_page_faults = 0]
       init_is_first_change[is_first_change = true]
   end
   
   subgraph for[用 c 遍历 changes]
       direction TB
   
       输出分隔符 --> out_table --> 输出缺页情况 --> 计数[n_page_faults += !c.hit]
   
       subgraph 输出分隔符
           is_first_change
           -->|true| set_is_first_change[is_first_change = false]
           is_first_change
           -->|false| sep["输出 /"]
       end
   
       subgraph out_table["输出“页表”：用 i 遍历 c.table"]
           idle[i == IDLE] -->|"✓"| 输出- --> 输出,
           idle -->|"✗"| 输出i --> 输出,
       end
       
       输出缺页情况["输出缺页情况：输出 c.hit ? 1 : 0"]
   end
   ```

### 页面置换策略设计

## 实验结果及数据分析

## 总结

## 附录

程序清单及说明。（列出文件名及说明即可，不需要在此处复制代码，代码直接以源文件形式提供，但源文件中对代码要有必要的注释和说明）
