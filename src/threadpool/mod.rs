// TODO: Concertar problema de uso excessivo de memória

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
pub struct Worker {
	_id: usize,
	thread: Option<thread::JoinHandle<()>>,
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
	#[warn(dead_code)]
	pub fn new(id: usize, reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
		let thread = thread::spawn(move || loop {
			let message = reciver.lock().unwrap().recv();
			match message {
				Ok(job) => job(),
				Err(_) => break,
			}
		});

		Worker {
			_id: id,
			thread: Some(thread),
		}
	}
}

pub struct ThreadPool {
	threads: Vec<Worker>,
	sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
	pub fn new(size: usize) -> ThreadPool {
		assert!(size > 0);
		let (sender, rec) = mpsc::channel();
		let reciver = Arc::new(Mutex::new(rec));
		let mut workers = Vec::with_capacity(size);

		for id in 0..size {
			workers.push(Worker::new(id, Arc::clone(&reciver)));
		}

		ThreadPool {
			threads: workers,
			sender: Some(sender),
		}
	}

	pub fn execute<F>(&self, f: F)
	where
		F: FnOnce() + Send + 'static,
	{
		let job = Box::new(f);

		if let Some(l) = self.sender.as_ref() {
			l.send(job).unwrap();
		}
	}

	// basicamente para quanod ele for terminar ele "limpar" todas as threas
	// insere funções vazias, sem ele basicamente o desligar demora mais
	pub fn finish(&self) {
		for _i in 0..self.threads.len() {
			if let Some(l) = self.sender.as_ref() {
				let _ = l.send(Box::new(|| {}));
			}
		}
	}
}

impl Drop for ThreadPool {
	fn drop(&mut self) {
		drop(self.sender.take());
		for worker in &mut self.threads {
			if let Some(thread) = worker.thread.take() {
				thread.join().unwrap();
			}
		}
	}
}
