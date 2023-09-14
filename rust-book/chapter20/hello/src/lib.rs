use std::fmt;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread; // multi-producer single consumer.

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type SharedReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;

impl Worker {
    fn new(id: usize, receiver: SharedReceiver) -> Worker {
        let thread = Some(thread::spawn(move || loop {
            let maybe_job = receiver.lock().unwrap().recv();
            match maybe_job {
                Ok(job) => {
                    println!("worker {} is running job", id);
                    job();
                }
                Err(_) => {
                    println!("worker {} got error. Exitting", id);
                    break;
                }
            }
        }));
        return Worker { id, thread };
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, Clone)]
pub struct PoolCreationError {
    msg: String,
}

impl PoolCreationError {
    fn new(msg: String) -> PoolCreationError {
        return PoolCreationError { msg: msg };
    }
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to create pool: {}", self.msg)
    }
}

impl ThreadPool {
    pub fn build(count: usize) -> Result<ThreadPool, PoolCreationError> {
        if count == 0 {
            return Err(PoolCreationError::new(String::from(
                "expected non-zero pool size, got zero",
            )));
        }

        let (sender, receiver) = mpsc::channel();

        let shared_receiver = SharedReceiver::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(count);

        for id in 0..count {
            // create some threads and store them in the vector
            workers.push(Worker::new(id, shared_receiver.clone()));
        }

        let sender = Some(sender);

        return Ok(ThreadPool { workers, sender });
    }

    pub fn execute<Fn>(&self, f: Fn)
    where
        Fn: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender.
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);
            let owned_thread = worker.thread.take();
            if owned_thread.is_some() {
                let owned_thread = owned_thread.unwrap();
                owned_thread.join().unwrap();
            }
        }
    }
}

#[test]
fn test_thread_pool() {
    let got = ThreadPool::build(0);
    assert!(!got.is_ok());
    println!("got: {}", got.err().unwrap());
}

#[test]
fn test_thread_pool_drop() {
    let tp = ThreadPool::build(2).unwrap();
    tp.execute(|| {});
    tp.execute(|| {});
    drop(tp);
}
