use std::{net::TcpListener, sync::{Arc, atomic::AtomicBool, atomic::Ordering::SeqCst}};
use std::env;
// mod threadpool;

mod server;
use server::*;

use std::process::Command;

// use std::sync::{atomic::{AtomicBool, Ordering::SeqCst}};
use ctrlc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        panic!("Porta não informada");
    }
    let ip_porta = format!("127.0.0.1:{}", &args[1]);
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    let _ = ctrlc::set_handler(move || {
        r.store(false, SeqCst);
        // println!("Desligando");
       let _ = Command::new("curl").arg("localhost:8000").output();
    });

    let lister = TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"); 
    while running.load(SeqCst) {
    match lister.accept(){
            Ok((mut s, _)) => {
                handle_con(&mut s);
            },
            _ => {
                // n deu
            }
        }
    // for s in lister.incoming() {
    //     let mut stream = s.unwrap();
    //     handle_con(&mut stream);
    // }
    }
    println!("Desligando");

}
