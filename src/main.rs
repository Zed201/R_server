use std::env;
use std::io::Write;
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
use protocol::Role;
use server::{log::shutdown, *};

use ctrlc;

use std::thread::{self, spawn};

use tungstenite::*;

use std::fs::File;
use std::io::prelude::*;

fn test() {
	/*	Da para fazer assim colocando apenas esse html no arquivo
	<script>
		const s = new WebSocket("ws://127.0.0.1:8000")
		console.log(s)
		s.addEventListener("message", (event) => {
			location.reload()
		});

	</script>
	 */
	/* rodou por 52s e vazou 83k de memoria, pode funcionar assim, mas tem de resolver o leak
	exemplo de criar websocket, mas precisa partir do navegador */
	let server = TcpListener::bind("127.0.0.1:8000").unwrap();
	for stream in server.incoming() {
		let mut s = stream.unwrap();
		println!("iwebf");
		// let _ = s.write(String::from("<h1>Titulo</h1>").as_bytes()).unwrap();
		// let p = read_req(&mut s);
		let r = String::from("index.html"); // provisorio so para testar
		const FILE_SOURCE_PATH: &str = "./test_source/";
		handle_con(&mut s);
		// if i % 2 == 0 {
		spawn(move || {
			// let mut websocket = accept(stream.unwrap()).unwrap();
			// let mut websocket = WebSocket::from_raw_socket(stream, Role::Server, None);

			// println!("Conecxão feita");
			//     // let msg = websocket.read().unwrap();
			// 	thread::sleep(Duration::from_secs(5));
			// 	websocket.send(Message::Text(String::new())).unwrap();
			// 	println!("Enviou");
			// assim ele fica mandando o navegador dar reload(olhar o codigo do live server para ver como fazer isso de forma melhor)
			let server_so = TcpListener::bind("127.0.0.1:8001").unwrap();
			let mut file = File::open(format!("{}{}", FILE_SOURCE_PATH, r)).unwrap();
			let mut last = file.metadata().unwrap().modified().unwrap();
			let mut b = false;
			loop {
				if !b {
					let s2 = server_so.accept().unwrap().0;
					let mut web = accept(s2).unwrap();
					let _ = web.send(Message::Text(String::new()));
					b = true;
				}
				thread::sleep(Duration::from_secs(1));
				let x = file.metadata().unwrap().modified().unwrap();
				if last != x{
					last = x;
					b = false;
				}
			
			}
		});
		// }
	}
}

fn main() {
	test();
	// let args: Vec<String> = env::args().collect();
	// if args.len() < 2 {
	// 	panic!("Porta não informada");
	// }
	// let ip_porta = format!("0.0.0.0:{}", &args[1]);
	// // 0.0.0.0 para ele se conectar a todas as placas de rede
	// // sejam virtuais ou físicas do sistema
	// on();
	// let running = Arc::new(AtomicBool::new(true));
	// let r = Arc::clone(&running);

	// let _ = ctrlc::set_handler(move || {
	// 	r.store(false, SeqCst);
	// });

	// // let _ = thread::spawn(move || {
	// //     thread::sleep(Duration::from_secs(5));
	// //     r.store(false, SeqCst);
	// // });

	// let lister = Arc::new(
	// 	TcpListener::bind(ip_porta.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"),
	// );
	// lister.set_nonblocking(true).unwrap();

	// // /*
	// //  * Basicamente designando cada thread da pool para lidar com uma das requisições
	// //  * cada uma delas tem um pequeno leak, com 10 threads teve um leak de 3.87k
	// //  * com 30 teve 6.69k(não é tão proporcional), mas independente do tempo de
	// //  *  testes o leak é sempre esse mesmo
	// //  */
	// // /*
	// //  * Por alfum motivo o programa fica consumindo memória até ser morto pelo sistema
	// //  * mas o mais estranho é que ele funciona normal, como funciona no cin, quando ta
	// //  * rodando usando o heapstack para funcionar, o que ta usando thread::spawn ele funciona
	// //  * normal, deixa mais memória sem liberar, mas não é morto pelo sistema, mesmo
	// //  * tendo muitas e muitas requisições
	// //  * */
	// // ! Essa solução com 5 threads ela tem um total de 1.76k de memoria leaked
	// let num_threads = 5;
	// for _i in 1..num_threads {
	// 	let l = lister.clone();
	// 	let r = running.clone();
	// 	thread::spawn(move || {
	// 		loop {
	// 			match l.accept() {
	// 				Ok((mut s, _)) => {
	// 					handle_con(&mut s);
	// 				}
	// 				_ => {
	// 					// nada
	// 				}
	// 			}
	// 			if !r.load(SeqCst) {
	// 				break;
	// 			}
	// 		}
	// 	});
	// }
	// while running.load(SeqCst) {}
	// thread::sleep(Duration::from_secs(1));
	// shutdown();
}
