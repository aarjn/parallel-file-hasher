# Parallel File Hasher

A tool to find duplicate files by hashing them in parallel. Built to understand Rust threading concepts.

## What It Does

Scans a directory, computes SHA256 hash of every file using multiple worker threads, and reports duplicates.

```bash
cargo run files

# Output:
# Hashed DirEntry("files/jackfruit.jpeg") on ThreadId(2), Hash: "e4717ec06e5fe05650da538c03f0ac08b428ea49bd406b1ec4b8bedcdf025d71"
# Hashed DirEntry("files/orange2.txt") on ThreadId(2), Hash: "be6fe11876282442bead98e8b24aca07f8972a763cd366c56b4b5f7bcdd23eac"
# Hashed DirEntry("files/apple.txt") on ThreadId(2), Hash: "303980bcb9e9e6cdec515230791af8b0ab1aaa244b58a8d99152673aa22197d0"
# Hashed DirEntry("files/orange.txt") on ThreadId(2), Hash: "be6fe11876282442bead98e8b24aca07f8972a763cd366c56b4b5f7bcdd23eac"

# Duplicate (hash: be6fe11876282442...):
#  files/orange2.txt
#  files/orange.txt

# All jobs submitted, dropping pool
```

## Architecture

```
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

Instead of spawning a new thread for every file (expensive), we reuse a fixed number of worker threads.

## Core Concepts

### Threading vs Concurrency

**Threading** is a mechanism - actual OS threads running in parallel on multiple CPU cores.

**Concurrency** is a concept - managing multiple tasks that can make progress independently.

```
Threading:     Two cooks, two stoves, cooking simultaneously
Concurrency:   One cook, two stoves, switching between pots
```

This project uses both: multiple threads (threading) sharing a job queue (concurrency).

### The Job Type

```rust
type Job = Box<dyn FnOnce() + Send + 'static>;
```

| Part | Meaning |
|------|---------|
| `Box<...>` | Heap allocation. Closures have unknown size, Box gives them a fixed size (pointer) |
| `dyn` | Dynamic dispatch. "Some type that implements these traits, figured out at runtime" |
| `FnOnce()` | A closure that can only be called once (may consume captured values) |
| `Send` | Safe to transfer to another thread |
| `'static` | Owns all its data, doesn't borrow anything that could dangle |

### Arc and Mutex - Sharing Data Between Threads

**Problem:** One channel receiver, multiple workers need to pull from it.

```rust
let (sender, receiver) = mpsc::channel();
let receiver = Arc::new(Mutex::new(receiver));
```

**`Arc`** (Atomic Reference Counting) - Multiple owners can share the same data. Each clone increments a counter. When all clones drop, data is freed.

```rust
let receiver = Arc::clone(&receiver);  // Cheap pointer copy, not data copy
```

**`Mutex`** (Mutual Exclusion) - Only one thread can access the data at a time. Others wait.

```rust
let job = receiver
    .lock()      // Block until we get exclusive access
    .unwrap()    // Handle poison (panic in another thread)
    .recv();     // Now safe - we're the only one
```

Together:

```
Arc<Mutex<Receiver<Job>>>
 │    │      │
 │    │      └── The actual data
 │    └── Only one thread touches it at a time
 └── Multiple threads can hold a pointer to it
```

### The Worker Loop

```rust
let thread = thread::spawn(move || {
    loop {
        match receiver.lock().unwrap().recv() {
            Ok(job) => job(),   // Got a job, execute it
            Err(_) => break,    // Channel closed, exit loop
        }
    }
});
```

Workers loop forever, waiting for jobs. When the channel closes (sender dropped), `recv()` returns `Err` and they exit cleanly.

### Why Drop

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());  // Close channel

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();  // Wait for thread to finish
            }
        }
    }
}
```

`Drop` is Rust's destructor - called automatically when a value goes out of scope. Our implementation:

1. Drops the sender → channel closes
2. Workers see `Err` from `recv()` → break their loops
3. `join()` waits for each worker to finish

### Saving Results

```rust
let results: Arc<Mutex<HashMap<String, Vec<String>>>> = 
    Arc::new(Mutex::new(HashMap::new()));
```

Same pattern: `Arc` for shared ownership across threads, `Mutex` for safe mutation.

```rust
pool.execute(move || {
    if let Ok(hash) = hash_file(&path) {
        let mut map = results.lock().unwrap();
        map.entry(hash)
            .or_insert_with(Vec::new)
            .push(path_str);
    }
});
```

Each worker hashes a file and adds it to the shared hashmap. Files with identical hashes end up in the same Vec.

## Files

```
src/
  main.rs          # CLI, spawns pool, collects results
  thread_pool.rs   # Thread pool implementation
  hasher.rs        # SHA256 file hashing
```

