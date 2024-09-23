#![allow(unused_imports)]
// organizar esses imports
use std::env;
use std::io::Write;
use std::time::Duration;
use std::{
    net::{Shutdown, TcpListener},
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
};

use log::on;

mod server;
// use protocol::Role;
use server::{log::shutdown, *};

use ctrlc;
use std::collections::HashMap;
use std::sync::mpsc::{self, channel};

use std::thread::{self, spawn};

use tungstenite::{protocol::Role, *};

use std::fs::File;
use std::io::prelude::*;

use std::path::*;
use std::time::SystemTime;

use clap::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Mode{
    Web,
    Live
}

static MODE_STR: &str = "Escolha entre os modos web(servidor http normal) e o \nlive(servidor funcionando como live server)\n";
static PORT_STR: &str = "Escolha da porta na qual o servidor vai ficar ouvindo, \nse for escolhido o modo live, o servidor vai ficar na 'porta' \nescolhida e o websocket vai ficar na 'porta + 1'\n";

fn main() {
    // so peguei do gpt
    let cmd = Command::new("R_server")
        .args(&[
            Arg::new("port_no_flag")
                .help(PORT_STR)
                .required(false)
                .value_parser(clap::value_parser!(u16))
                .index(1),
            Arg::new("port")
                .short('p')
                .long("port")
                .help(PORT_STR)
                .required(false)
                .value_parser(clap::value_parser!(u16)),

            Arg::new("mode_no_flag")
                .help(MODE_STR)
                .required(false)
                .value_parser(clap::builder::EnumValueParser::<Mode>::new())
                .index(2),
            Arg::new("mode")
                .short('m')
                .long("mode")
                .help(MODE_STR)
                .required(false)
                .value_parser(clap::builder::EnumValueParser::<Mode>::new())

        ]).get_matches();

    let porta = cmd.get_one::<u16>("port")
        .or_else(|| cmd.get_one::<u16>("port_no_flag"))
        .cloned()
        .unwrap_or(8000);
    let mode = cmd.get_one::<Mode>("mode")
        .or_else(|| cmd.get_one::<Mode>("mode_no_flag"))
        .cloned()
        .unwrap_or(Mode::Web);

    let ip = format!("0.0.0.0:{}", porta);
    // 0.0.0.0 para ele se conectar a todas as placas de rede
    // sejam virtuais ou físicas do sistema

    /* o segundo argumento será o "tipo" de servidor, web normal ou live(com reload)
    * O normal será web e o outro live
    */
    on();
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    let _ = ctrlc::set_handler(move || {
        r.store(false, SeqCst);	
    });

    // // let h = thread::spawn(move || { // para testes de tempo
    // //     thread::sleep(Duration::from_secs(50));
    // //     r.store(false, SeqCst);
    // // });

    let lister =
    Arc::new(TcpListener::bind(ip.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"));
    // mesmo setando isso os navegadores ainda conseguem bloquear
    lister.set_nonblocking(true).unwrap();

    match mode {
        Mode::Live => {
            live_server(lister, running, porta);
        }
        Mode::Web => {
            normal_server(lister, running);
        }
    }

    thread::sleep(Duration::from_secs(1));
    // h.join();
    shutdown();
}

fn normal_server(lister: Arc<TcpListener>, running: Arc<AtomicBool>) {
    // ! com 5 ele deixa leaked apenas 1.76k independente do tempo
    // Com 2 ou com 1 funciona, mas fica travado a apenas 1 navegador, por causa do navegador
    // travar uma conexão
    let num_threads = 5;
    let mut handles = Vec::with_capacity(num_threads);
    for _i in 1..num_threads {
        let l = lister.clone();
        let r = running.clone();
        let h = thread::spawn(move || {
            loop {
                match l.accept() {
                    Ok((mut s, _)) => {
                        handle_con(&mut s);
                        s.shutdown(Shutdown::Both).expect("Falha ao fechar conexão");
                    }
                    _ => {
                        // nada
                    }
                }
                if !r.load(SeqCst) {
                    break;
                }
            }
        });
    handles.push(h);
    }
    while running.load(SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }
    // se tirar o for ao sair ele obviamente não trava mas deixa vazamento de memoria
    for h in handles {
    h.join().unwrap();
    }
}

fn live_server(lister: Arc<TcpListener>, running: Arc<AtomicBool>, porta: u16) {
    let (tx, rx) = channel::<u8>();

    let r = Arc::clone(&running);
    let p = thread::spawn(move || {
        let websocket = TcpListener::bind(format!("0.0.0.0:{}", porta + 1)).unwrap();
        loop {
            let _ = rx.recv();
            println!("Ping");
            let s = websocket.accept().unwrap().0;
            let mut w = tungstenite::accept(s).unwrap();
            let _ = w.send(tungstenite::Message::Text(String::new())).unwrap();
            if !r.load(SeqCst) {
                break;
            }
        }
    });
    // vai ter o nome do arquivo(desconsiderar pastas por enquanto) e o tempo de ultima modificação
    let mut set: HashMap<String, SystemTime> = HashMap::new();
    // não vai ser multi threading, no momento ainda estou vendo certinho como vai funcionar
    loop {
        if !running.load(SeqCst) {
            break;
        }
        match lister.accept() {
            Ok((mut s, _)) => {
                soc_con(&mut s, &mut set);
                s.shutdown(Shutdown::Both).unwrap();
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(150)); // espera um pouco
        for (nome, time_) in set.iter_mut() {
            let x = &format!("{}{}", FILE_SOURCE_PATH, nome);
            let p = Path::new(x);
            let c = p.metadata().unwrap().modified().unwrap();
            if c != *time_ {
                *time_ = c;
                tx.send(1).unwrap();
            }
        }
    }
p.join().unwrap();
}
