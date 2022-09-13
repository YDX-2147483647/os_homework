"""
https://www.jianshu.com/p/6e415fdce561
Accepted except #7.
Python 2
"""


class Thread:
    id, arrive, cost, priority, quantum, done = 0, 0, 0, 0, 0, False

    def __init__(self, param):
        self.id = param[0]
        self.arrive = int(param[1])
        self.cost = int(param[2])
        self.priority = int(param[3])
        self.quantum = int(param[4])


def inputThread():
    threads, num = [], 0
    while True:
        try:
            threads.append(Thread(raw_input().split("/")))
            # For python 3
            # threads.append(Thread(input().split("/")))
            num += 1
        except:
            break
    return threads, num


def output(logidx, tid, tstart, tend, tpriority):
    print str(logidx) + '/' + str(tid) + '/' + str(tstart) + '/' + str(tend) + '/' + str(tpriority)
    # For python 3
    # print(str(logidx) + '/' + str(tid) + '/' + str(tstart) +
    #       '/' + str(tend) + '/' + str(tpriority))


def cmp_arrive_id(x, y):
    # if int(x.arrive) == int(y.arrive):
    #     return int(x.id) - int(y.id)
    return int(x.arrive) - int(y.arrive)


def dp():
    threads, num = inputThread()
    threads.sort(cmp_arrive_id)
    # For python 3
    # threads.sort(key=lambda x: x.arrive)
    time, last, done_count, idx = 0, 0, 0, 1

    while done_count < num:
        i = 0
        while i < num and threads[i].arrive <= time:
            # new tasks arrived in last quantum (exclusively)
            if last < threads[i].arrive < time:
                threads[i].priority = max(threads[i].priority - 1, 0)
            i += 1

        # 2. Get next task from `ready_tasks`
        minimum, arg_min = 99999999999999999999, -1
        # range(i): all arrived tasks
        for j in range(i):
            # lord algorithm among ready tasks
            if threads[j].priority < minimum and not bool(threads[j].done):
                minimum, arg_min = threads[j].priority, j
        # If nothing ready
        if arg_min == -1:
            # skip to next arrival and try again
            time = threads[i].arrive
            continue
        last = time

        # 4. Run it
        # 4.0 Update tasks' priorities
        for j in range(i):
            if not bool(threads[j].done):
                if j == arg_min:
                    threads[j].priority += 3
                else:
                    threads[j].priority = max(threads[j].priority - 1, 0)
        # 4.1 Calculate how long it will run.
        duration = min(threads[arg_min].cost, threads[arg_min].quantum)
        # 4.2 Update the schedule
        output(idx, threads[arg_min].id, time, time +
               duration, threads[arg_min].priority)
        # 4.3 Let time fly.
        time += duration
        threads[arg_min].cost -= duration

        # Count
        if threads[arg_min].cost == 0:
            done_count += 1
            threads[arg_min].done = True
        idx += 1


if __name__ == '__main__':
    method = input()
    assert method == 5
    # For python 3
    # assert int(method) == 5
    dp()
