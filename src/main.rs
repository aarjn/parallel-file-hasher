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
}
