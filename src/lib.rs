use std::fmt;
use std::thread;
use std::vec::Vec;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    // /// The `new` function will panic if the size is zero.
    // pub fn new(size: usize) -> ThreadPool {
    //     assert!(size > 0);

    //     ThreadPool {}
    // }

    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        let (sender, receiver) = channel::<Message>();
        let rx = Arc::new(Mutex::new(receiver));

        if size > 0 {
            let mut workers = Vec::with_capacity(size);

            for idx in 0..size {
                workers.push(Worker::new(idx, Arc::clone(&rx)));
            }

            Ok(ThreadPool {
                workers,
                sender,
            })
        } else {
            Err(PoolCreationError { message: "ThreadPool size has to be greater than zero!".to_string()})
        }
    }

    pub fn execute<F>(&self, handler: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(handler);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }


        println!("Shutting down all workers.");
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub struct PoolCreationError {
    message: String,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PoolCreationError: {}", &self.message)
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            Worker::work(id, receiver)
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }

    fn work(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) {
        loop {
            let message = {
                let receiver = receiver.lock().unwrap();

                match receiver.recv() {
                    Ok(message) => message,
                    Err(_) => {
                        eprintln!("Worker [{}] channel closed", id);

                        break;
                    },
                }
            };

            if let Message::NewJob(job) = message {
                println!("Worker [{}] got a job; executing", id);

                job();
            } else {
                println!("Worker [{}] was told to terminate", id);

                break;
            }
        }
    }
}
