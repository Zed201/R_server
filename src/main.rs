use std::{net::TcpListener, sync::{atomic::{AtomicBool, Ordering::SeqCst}, Arc}, thread};
use std::env;
mod threadpool;
use threadpool::*;

mod server;
use server::*;

use std::io::{prelude::*, BufReader};
use std::process::Command;

use std::collections::HashMap;
// use std::sync::{atomic::{AtomicBool, Ordering::SeqCst}};
use ctrlc;
// use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        panic!("Porta não informada");
    }
    let ip_porta = format!("127.0.0.1:{}", &args[1]);
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    // let pool = ThreadPool::new(10);
    let _ = ctrlc::set_handler(move || {
        r.store(false, SeqCst);
        let _ = Command::new("curl")
            .arg("-A")
            .arg("END")
            .arg("localhost:8000")
            .output()
            .expect("Deu ruim ao executar o curl"); // envia uma requisição
        println!("Já foi");
    });

    let lister = TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"); 
    lister.set_nonblocking(true).unwrap();
    // alguma coisa de pausa de thread ta acontecendo por aqui, pois sem o threadpool o 
    // curl para de pegar apos ter alguma requisição do navegador normal, mas e nao tiver nenhuma
    // ele funciona normal para liberar memóri
    while running.load(SeqCst) {
        match lister.accept(){
            Ok((mut s, _)) => {
                // println!("----");
                handle_con(&mut s);
                s.shutdown(Shutdown::Both).expect("Erro ao fechar conexão");
                
            },
            _ => {
                // nada
            }
        }
    }
}
