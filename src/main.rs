pub mod hasher;
pub mod thread_pool;

use hasher::hash_file;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thread_pool::ThreadPool;
use walkdir::WalkDir;

fn main() {
    let dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let pool = ThreadPool::new(4);

    // shared hashmap: hash -> list of file paths
    let results: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();

        if path.is_file() {
            let results = Arc::clone(&results);

            pool.execute(move || {
                if let Ok(hash) = hash_file(&path) {
                    let path_str = path.to_string_lossy().to_string();
                    let mut map = results.lock().unwrap();
                    println!(
                        "Hashed {:?} on {:?}, Hash: {:?}",
                        entry,
                        thread::current().id(),
                        hash
                    );
                    map.entry(hash).or_insert_with(Vec::new).push(path_str);
                }
            });
        }
    }

    drop(pool);

    let map = results.lock().unwrap();
    for (hash, paths) in map.iter() {
        if paths.len() > 1 {
            println!("\nDuplicate (hash: {}...):", &hash[..16]);
            for p in paths {
                println!("  {}", p);
            }
        }
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
