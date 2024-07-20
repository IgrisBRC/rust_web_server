use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            handle: Some(thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job, executing");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected, shutting down");
                        break;
                    }
                }
            })),
        }
    }
}

impl ThreadPool {
    /// Creating a new ThreadPool
    /// size is the number of threads in the thread pool
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if size is zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

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

        if let Some(sender) = &self.sender {
            sender.send(job).unwrap();
        }

        //self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(std::mem::replace(&mut self.sender, None));

        for worker in &mut self.workers {
            println!("Worker {} shutting down", worker.id);

            std::mem::replace(&mut worker.handle, None)
                .expect("Couldn't shutdown thread gracefully")
                .join()
                .unwrap();
        }
    }
}
