use std::env;
use std::time::Duration;
use std::{
	net::TcpListener,
	sync::{
		atomic::{AtomicBool, Ordering::SeqCst},
		Arc,
	},
};

mod threadpool;
use log::on;

mod server;
use server::{log::shutdown, *};

use ctrlc;

use std::thread;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		panic!("Porta não informada");
	}
	let ip_porta = format!("0.0.0.0:{}", &args[1]);
	// 0.0.0.0 para ele se conectar a todas as placas de rede
	// sejam virtuais ou físicas do sistema
	on();
	let running = Arc::new(AtomicBool::new(true));
	let r = Arc::clone(&running);

	let _ = ctrlc::set_handler(move || {
		r.store(false, SeqCst);
	});

	// let _ = thread::spawn(move || {
	//     thread::sleep(Duration::from_secs(5));
	//     r.store(false, SeqCst);
	// });

	let lister = Arc::new(
		TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"),
	);
	lister.set_nonblocking(true).unwrap();

	// /*
	//  * Basicamente designando cada thread da pool para lidar com uma das requisições
	//  * cada uma delas tem um pequeno leak, com 10 threads teve um leak de 3.87k
	//  * com 30 teve 6.69k(não é tão proporcional), mas independente do tempo de
	//  *  testes o leak é sempre esse mesmo
	//  */
	// /*
	//  * Por alfum motivo o programa fica consumindo memória até ser morto pelo sistema
	//  * mas o mais estranho é que ele funciona normal, como funciona no cin, quando ta
	//  * rodando usando o heapstack para funcionar, o que ta usando thread::spawn ele funciona
	//  * normal, deixa mais memória sem liberar, mas não é morto pelo sistema, mesmo
	//  * tendo muitas e muitas requisições
	//  * */
	// ! Essa solução com 5 threads ela tem um total de 1.76k de memoria leaked
	let num_threads = 5;
	for _i in 1..num_threads {
		let l = lister.clone();
		let r = running.clone();
		thread::spawn(move || {
			loop {
				match l.accept() {
					Ok((mut s, _)) => {
						handle_con(&mut s);
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
	}
	while running.load(SeqCst) {}
	thread::sleep(Duration::from_secs(1));
	shutdown();
}
