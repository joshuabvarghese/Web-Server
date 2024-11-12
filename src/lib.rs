use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

// Define the ThreadPool struct, which will manage a set of worker threads.
pub struct ThreadPool {
    // Holds the list of workers in the pool.
    workers: Vec<Worker>,
    // The sender allows dispatching jobs to the workers. Option allows cleanup when dropped.
    sender: Option<mpsc::Sender<Job>>,
}

// Type alias for a "Job", which is any function or closure that takes no arguments and returns nothing.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new ThreadPool.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `size` is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Thread pool size must be greater than zero.");

        // Set up a channel to send jobs from the main thread to the worker threads.
        let (sender, receiver) = mpsc::channel();
        
        // Wrap the receiver in an Arc and Mutex for safe sharing among multiple threads.
        let receiver = Arc::new(Mutex::new(receiver));

        // Pre-allocate the workers vector to improve efficiency.
        let mut workers = Vec::with_capacity(size);

        // Create the specified number of workers and store them in the pool.
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Executes a job by sending it to one of the worker threads.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Wrap the closure in a Box to create a Job and send it through the channel.
        let job = Box::new(f);

        // Send the job to the workers via the sender channel.
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

// Implement Drop trait to ensure proper cleanup when ThreadPool goes out of scope.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close the sender, which signals the workers to stop receiving jobs.
        drop(self.sender.take());

        // Join each worker thread to ensure proper shutdown.
        for worker in &mut self.workers {
            println!("Worker {} shutting down.", worker.id);

            // Take the thread handle out of the Option and join on it.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Define a Worker struct that holds an ID and a thread.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker that waits for jobs on the receiver channel.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // Spawn a new thread for the worker.
        let thread = thread::spawn(move || loop {
            // Lock the receiver to receive a job, and handle any potential errors.
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} received a job; executing.");

                    job();
                }
                Err(_) => {
                    // Break out of the loop if the channel is closed, which indicates shutdown.
                    println!("Worker {id} disconnected; shutting down.");
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
