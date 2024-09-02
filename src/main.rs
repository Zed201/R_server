use std::io::{prelude::*, BufReader};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, thread};
use std::{net::TcpListener, net::TcpStream};

fn read_req(stream: &mut TcpStream) -> Vec<String> {
    let buffer = BufReader::new(stream);
    let request: Vec<String> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|linha| !linha.is_empty())
        .collect();
    request
}

fn code_to_status(code: u32) -> String {
    match code {
        200 => String::from("200 OK"),
        404 => String::from("404 NOT FOUND"),
        _ => String::from("200 Ok"),
    }
}

// fn cont_type(file_name: &str) -> {

// }

///////////////////////////////////////
/// A estrutura básica da resposta do http é 
/// HTTP/1.1 (code_status) (code_msg)
/// Content-Type: (Tipo de conteudo);
/// <body> EM BYTES 
///////////////////////////////////////
/// (Tipo de conteudo)(final com \r\n)
/// text/javascript
/// text/css
/// text/html
/// text/plain
/// image/x-icon
/// image/png // implementar apenas png
/// application/json
/// application/pdf

fn file_sender(stream: &mut TcpStream, status: u32, file: &str) -> Result<(), String> {
    let status_line = format!("HTTP/1.1 {}", code_to_status(status));
    if file.ends_with(".html") {
        let content = match fs::read_to_string(format!("src/{}", file)) {
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que é texto", file)),
        };

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\n\r\n{content}",
            content.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let content = match fs::read(format!("src{}", file)) {
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que não é texto", file)),
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/x-icon\r\nContent-Length: {}\r\n\r\n",
            content.len()
        );
        stream.write(response.as_bytes()).unwrap();
        stream.write(&content).unwrap();
    }
    Ok(())
}

fn handle_con(stream: &mut TcpStream) {
    let request = read_req(stream);
    let uri: Vec<&str> = request[0].split(" ").collect();

    // melhorar aqui, para redirecionar para um padrão de html
    if uri[0] == "GET" {
        println!("{:?}", uri);
        let file = if uri[1] != "/" { uri[1] } else { "index.html" };
        let _ = file_sender(stream, 200, file).unwrap();
    }
    thread::sleep(Duration::from_secs(5));
}

fn main() {
    let lister = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(10);
    for s in lister.incoming() {
        let mut stream = s.unwrap();

        pool.execute(move || handle_con(&mut stream));
    }
    println!("Desligando");
}

// ThreadPool

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
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

struct ThreadPool {
    threads: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
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

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(l) = self.sender.as_ref() {
            l.send(job).unwrap();
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
