use anyhow::Result;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

// =======================================================================================================

pub type Job = Box<dyn FnOnce() -> Result<()> + Send + 'static>;

// =======================================================================================================

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, reveiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(reveiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id,  Arc::clone(&receiver)));
        }

        Self { workers, sender: Some(sender) }
    }

    pub fn push_job(&self, job: Job) -> Result<()> {
        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .map_err(|e| anyhow::anyhow!("Failed to send job to worker: {}", e))
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            println!("Waiting for worker {} to shut down", worker.id);
            if let Err(e) = worker.handle.join() {
                eprintln!("Worker {} panicked: {:?}", worker.id, e);
            }
        }
    }
}

// =======================================================================================================

struct Worker {
    id: usize,
    handle: std::thread::JoinHandle<()>,
}

impl Worker {
    fn new (id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {        
            let handle = thread::Builder::new()
                .name("worker".to_string())
                .spawn(move || Worker::run(id, receiver))
                .unwrap();

            Self {id, handle }
    }

    fn run(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) {        
        println!("Worker thread {id} started");

        loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker thread {id} got a job; executing.");
                    let _ = job();
                }
                Err(_) => {
                    println!("Worker thread {id} disconnected; shutting down.");
                    break;
                }
            };
        }

        println!("Worker thread {id} exit");
    }
}
