# Parallel File Hasher

aim: to understand threads better. 

```
A thread pool is a set of worker threads that sit waiting for jobs. Instead of spawning a new thread for every task (expensive), you reuse a fixed number of threads.

                          ┌─────────────────────────────────────────┐
                          │             THREAD POOL                 │
                          │                                         │
   Job 1 ──┐              │   ┌──────────┐                          │
           │              │   │ Worker 0 │ ◄─── pulls job, executes │
   Job 2 ──┼──► [Queue] ──┼──►├──────────┤                          │
           │              │   │ Worker 1 │ ◄─── pulls job, executes │
   Job 3 ──┘              │   ├──────────┤                          │
                          │   │ Worker 2 │ ◄─── pulls job, executes │
                          │   └──────────┘                          │
                          │                                         │
                          └─────────────────────────────────────────┘
```
