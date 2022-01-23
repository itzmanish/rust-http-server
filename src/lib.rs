use std::sync::{mpsc, Arc, Mutex};
use std::thread;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = reciever.lock().unwrap().recv().unwrap();
            println!("worker(id:{}) got a job; executing:", id);
            job();
        });
        Worker { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub type ThreadPoolError = String;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> Result<ThreadPool, ThreadPoolError> {
        if size <= 0 {
            return Err("Error on creating pool".to_string());
        };
        let (sc, rc) = mpsc::channel();
        let rc = Arc::new(Mutex::new(rc));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rc)));
        }
        Ok(ThreadPool {
            workers,
            sender: sc,
        })
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
