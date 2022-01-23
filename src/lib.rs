use std::sync::{mpsc, Arc, Mutex};
use std::thread;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = reciever.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("worker(id:{}) got a job; executing:", id);
                    job();
                }
                Message::Terminate => {
                    println!(
                        "Worker(id:{}) got terminating signal. Closing worker....",
                        id
                    );
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub type ThreadPoolError = String;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("sending terminating signal to all active worker...");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

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
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}
