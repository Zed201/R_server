use std::net::TcpListener;

mod threadpool;
use threadpool::*;

mod server;
use server::*;

fn main() {
    let lister = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(10);
    for s in lister.incoming() {
        let mut stream = s.unwrap();

        pool.execute(move || handle_con(&mut stream));
    }
    println!("Desligando");

    
}