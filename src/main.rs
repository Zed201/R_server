use std::net::TcpListener;
use std::env;

mod threadpool;
use threadpool::*;

mod server;
use server::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        panic!("Porta não informada");
    }
    let lister = TcpListener::bind(format!("127.0.0.1:{}", &args[1])).expect("Não conseguiu criar o socket na porta escolhida\n");

    let pool = ThreadPool::new(10);
    for s in lister.incoming() {
        let mut stream = s.unwrap();

        pool.execute(move || handle_con(&mut stream));
    }
    println!("Desligando");

    
}
