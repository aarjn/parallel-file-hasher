pub mod hasher;
pub mod thread_pool;

use std::thread;
use std::time::Duration;
use thread_pool::ThreadPool;

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            println!("Job {} starting on {:?}", i, thread::current().id());
            thread::sleep(Duration::from_millis(500));
            println!("Job {} done", i);
        });
    }

    println!("All jobs submitted, dropping pool");

    // <-- RIGHT HERE: main() ends, `pool` goes out of scope
    // rust automatically calls `drop(&mut pool)`
    // your Drop implementation runs:
    //   1. Closes the channel
    //   2. Waits for all workers to finish
    // without Drop: program exits immediately, jobs might not finish
    // with Drop: program waits here for all 8 jobs to complete
}
