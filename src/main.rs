use std::{net::TcpListener};
use std::env;
// mod threadpool;

mod server;
use server::*;

// use std::sync::{atomic::{AtomicBool, Ordering::SeqCst}};
// use ctrlc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        panic!("Porta não informada");
    }
    let ip_porta = format!("127.0.0.1:{}", &args[1]);
    // let running = Arc::new(AtomicBool::new(true));
    // let r = Arc::clone(&running);

    let lister = TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"); 
    for s in lister.incoming() {
        let mut stream = s.unwrap();
        handle_con(&mut stream);
    }
    println!("Desligando");

}
