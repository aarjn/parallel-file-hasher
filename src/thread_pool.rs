use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // Arc and Mutex is used because one channel receiver, multiple worker threads need to pull
        // from it. Arc allow multiple owner share the same data. Mutex to only one thread can
        // access data at a time

        let receiver = Arc::new(Mutex::new(receiver));

        //Arc<Mutex<Receiver<Job>>>
        // │    │      │
        // │    │      └── The actual data
        // │    └── Only one thread touches it at a time
        // └── Multiple threads can hold a pointer to it

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                //let job = receiver
                //    .lock() // block untill we get exclusive access
                //    .unwrap() // handle panics
                //    .recv() // method used to receive data from a communication channel or a network socket
                //    .unwrap();

                match receiver.lock().unwrap().recv() {
                    Ok(job) => job(),
                    Err(_) => break,
                }

                //job();
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
