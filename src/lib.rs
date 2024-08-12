//! A thread pool library.
//!
//! This library provides a thread pool that can execute jobs concurrently.
use std::{
    thread,
    sync::{
        Arc,
        Mutex,
        mpsc
    }
};

/// Simple thread pool.
///
/// The thread pool will create a number of threads and execute jobs concurrently.
///
/// The thread pool will panic if the size is zero.
///
/// The thread pool will panic if the threads encounter an error while shutting down.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool{ workers, sender: Some(sender) }
    }

    /// Execute a job on the ThreadPool.
    ///
    /// The job must be a closure that takes no arguments and returns nothing.
    ///
    /// # Panics
    ///
    /// The `execute` function will panic if the thread pool has been dropped.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {

    /// Shut down the ThreadPool.
    ///
    /// The `drop` function will wait for all threads to finish executing their jobs.
    ///
    /// # Panics
    ///
    /// The `drop` function will panic if the threads encounter an error while shutting down.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(worker) = worker.thread.take() {
                worker.join().unwrap();
            }
        }
    }
}

/// A worker in the ThreadPool.
///
/// Each worker is associated with a thread that will execute jobs.
///
/// The worker will continue to execute jobs until it receives a termination signal.
///
/// The worker will panic if it encounters an error while receiving a job.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {

    /// Create a new Worker.
    ///
    /// The id is the worker's unique identifier.
    ///
    /// The receiver is the channel from which the worker will receive jobs.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the receiver encounters an error.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                },
                Err(_) => {
                    println!("Worker {} got a termination signal.", id);
                    return;
                }
            }
        });

        Worker{ id, thread: Some(thread) }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;