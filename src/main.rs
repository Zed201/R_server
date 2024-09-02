
use std::str::Bytes;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, thread};
use std::{net::TcpListener, net::TcpStream};
use std::io::{prelude::*, BufReader};

fn read_req(stream: &mut TcpStream) -> Vec<String>{
    let buffer =  BufReader::new( stream);
    let request: Vec<String> = buffer.
        lines().map(|result| result.unwrap()).take_while(|linha| !linha.is_empty()).collect();
    request
}

fn code_to_status(code: u32) -> String{
    match code {
        200 => String::from("OK"),
        404 => String::from("NOT FOUND"),
        _ => String::from("Ok"),
    }
}

fn file_sender(stream: &mut TcpStream, status: u32, file: &str) -> Result<(), String>{
    // println!("requisitando o arquivo {file}");
    let status_line = format!("HTTP/1.1 {} {}", status, code_to_status(status));
    if file.ends_with(".html"){

        let content = match fs::read_to_string(format!("src/{}", file)) {
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que é texto", file))
        };

        let response = format!("{status_line}\r\nContent-Length: {}\r\n\r\n{content}", content.len());
        stream.write_all(response.as_bytes()).unwrap();

    } else {
        let content = match fs::read(format!("src{}", file)){
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que não é texto", file)) 
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

struct worker{
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl worker{
    fn new(id: usize, reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> worker {
        let thread = thread::spawn(move || {
            while let Ok(job) = reciver.lock().unwrap().recv(){
                job();
            }
        });

        worker {id, thread}
    }
}

struct ThreadPool{
    threads: Vec<worker>,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, rec) = mpsc::channel();
        let reciver = Arc::new(Mutex::new(rec));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(worker::new(id, Arc::clone(&reciver)));
        }

        ThreadPool { threads: workers, sender }
    }

    fn execute<F>(&self, f:F)
    where 
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }

}

fn handle_con(stream: &mut TcpStream){
    let request =  read_req(stream);
    let uri: Vec<&str> = request[0].split(" ").collect();
        if uri[0] == "GET" {
            println!("{:?}", uri);
            let file =  if uri[1] != "/" {uri[1]} else {"index.html"};
            let _ = file_sender(stream, 200, file).unwrap();
        }
        thread::sleep(Duration::from_secs(5));
}

fn main() {
    let listenert = TcpListener::bind("127.0.0.1:8080").unwrap();
    
    let pool = ThreadPool::new(10);
    for s in listenert.incoming(){
        let mut stream = s.unwrap();
        
        
        pool.execute( move || {
            handle_con(&mut stream)
        });
        
    }

}
