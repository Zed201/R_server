#![allow(unused_imports)]

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

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 3 {
		// fazer um "default" para web depois
		panic!("Porta não informada");
	}
	let porta: u32 = args[1].parse().expect("Erro ao pegar a porta, (!não numérica)");
	let ip = format!("0.0.0.0:{}", porta);
	// 0.0.0.0 para ele se conectar a todas as placas de rede
	// sejam virtuais ou físicas do sistema
	// TODO: Melhorar a leitura de comandos, talvez adicionar uma lib para isso
	/* o segundo argumento será o "tipo" de servidor, web normal ou live(com reload)
	 * O normal	será --web e o outro --live
	 */
	on();
	let running = Arc::new(AtomicBool::new(true));
	let r = Arc::clone(&running);

	let _ = ctrlc::set_handler(move || {
		r.store(false, SeqCst);	
	});

	// let h = thread::spawn(move || { // para testes de tempo
	//     thread::sleep(Duration::from_secs(50));
	//     r.store(false, SeqCst);
	// });

	let lister =
		Arc::new(TcpListener::bind(ip.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"));

	lister.set_nonblocking(true).unwrap();

	match &args[2].as_str() {
		&"--live" => {
			live_server(lister, running, porta);
		}
		&"--web" => {
			normal_server(lister, running);
		}
		_ => {
			println!("Tipo não especificado")
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
	// for h in handles {
	//     h.join().unwrap();
	// }
}

fn live_server(lister: Arc<TcpListener>, running: Arc<AtomicBool>, porta: u32) {
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
