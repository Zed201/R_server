use std::{net::{Shutdown, TcpListener}, sync::{atomic::{AtomicBool, Ordering::SeqCst}, Arc}};
use std::env;
mod threadpool;
use threadpool::*;

mod server;
use server::*;

use std::io::{prelude::*};
use ctrlc;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        panic!("Porta não informada");
    }
    let ip_porta = format!("127.0.0.1:{}", &args[1]);
    
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    let pool = Arc::new(ThreadPool::new(10));
    let p = Arc::clone(&pool);

    let _ = ctrlc::set_handler(move || {
        p.finish();
        r.store(false, SeqCst);
    });

    let lister = Arc::new(TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n")); 
    lister.set_nonblocking(true).unwrap();

    // 1 thread para lidar com cada requisição, provavelmente cada thread está tendo algum leak, mas o programa sai certinho(! dps de um tempo mas sai)
    // TODO: Ou as vzes ele não vai depois de muitas requisições
    // TODO: Testar memory leak com valgrind
    while running.load(SeqCst) {
        let l = lister.clone();
        pool.execute(move || {
            match l.accept(){
                Ok((mut s, _)) => {
                    handle_con(&mut s);
                    s.flush().unwrap();
                    s.shutdown(Shutdown::Both).expect("Erro ao fechar conexão");
                },
                _ => {
                    // nada
                }
            }
        });
        
    }
}
