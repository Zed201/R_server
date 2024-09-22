#![allow(unused_imports)]

use std::env;
use std::io::Write;
use std::time::Duration;
use std::{
	net::{TcpListener, Shutdown},
	sync::{
		atomic::{AtomicBool, Ordering::SeqCst},
		Arc,
	},
};

use log::on;

mod server;
// use protocol::Role;
use server::{log::shutdown, *};

use std::collections::HashMap;
use std::sync::mpsc::{self, channel};
use ctrlc;

use std::thread::{self, spawn};

use tungstenite::{*, protocol::Role};

use std::fs::File;
use std::io::prelude::*;

use std::time::SystemTime;
use std::path::*;
// fn test_websocket() {
// 	/*	Da para fazer assim colocando apenas esse html no arquivo
// 	<script>
// 		const s = new WebSocket("ws://127.0.0.1:8000")
// 		console.log(s)
// 		s.addEventListener("message", (event) => {
// 			location.reload()
// 		});

// 	</script>
// 	 */
// 	/* rodou por 52s e vazou 83k de memoria, pode funcionar assim, mas tem de resolver o leak
// 	exemplo de criar websocket, mas precisa partir do navegador */
// 	let server = TcpListener::bind("127.0.0.1:8000").unwrap();
// 	for stream in server.incoming() {
// 		let mut s = stream.unwrap();
// 		println!("iwebf");
// 		// let _ = s.write(String::from("<h1>Titulo</h1>").as_bytes()).unwrap();
// 		// let p = read_req(&mut s);
// 		// let r = String::from("index.html"); // provisorio so para testar
// 		// const FILE_SOURCE_PATH: &str = "./test_source/";
// 		handle_con(&mut s);
// 		// println!("{}", read_req(&mut s)["required"]);
// 		// if i % 2 == 0 {
// 		// spawn(move || {
// 		// 	// let mut websocket = accept(stream.unwrap()).unwrap();
// 		// 	// let mut websocket = WebSocket::from_raw_socket(stream, Role::Server, None);

// 		// 	// println!("Conecxão feita");
// 		// 	//     // let msg = websocket.read().unwrap();
// 		// 	// 	thread::sleep(Duration::from_secs(5));
// 		// 	// 	websocket.send(Message::Text(String::new())).unwrap();
// 		// 	// 	println!("Enviou");
// 		// 	// assim ele fica mandando o navegador dar reload(olhar o codigo do live server para ver como fazer isso de forma melhor)
// 		// 	let server_so = TcpListener::bind("127.0.0.1:8001").unwrap();
// 		// 	let mut file = File::open(format!("{}{}", FILE_SOURCE_PATH, r)).unwrap();
// 		// 	let mut last = file.metadata().unwrap().modified().unwrap();
// 		// 	let mut b = false;
// 		// 	loop {
// 		// 		if !b {
// 		// 			let s2 = server_so.accept().unwrap().0;
// 		// 			let mut web = accept(s2).unwrap();
// 		// 			let _ = web.send(Message::Text(String::new()));
// 		// 			b = true;
// 		// 		}
// 		// 		thread::sleep(Duration::from_secs(1));
// 		// 		let x = file.metadata().unwrap().modified().unwrap();
// 		// 		if last != x {
// 		// 			last = x;
// 		// 			b = false;
// 		// 		}
// 		// 	}
// 		// });
// 		// }
// 	}
// }

fn main() {
	
	let args: Vec<String> = env::args().collect();
	if args.len() < 3 { // fazer um "default" para web depois
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
                // Arc::decrement_strong_count(r);
                // drop(r);
	});

	// let h = thread::spawn(move || { // para testes de tempo
	//     thread::sleep(Duration::from_secs(50));
	//     r.store(false, SeqCst);
	// });

	let lister = Arc::new(
		TcpListener::bind(ip.clone()).expect("Não conseguiu criar o socket na porta escolhida\n"),
	);

	lister.set_nonblocking(true).unwrap();
	match &args[2].as_str() {
		&"--live" => {
			live_server(lister, running, porta);
		},
		&"--web" => {
			normal_server(lister, running);
		},
		_ => {
			println!("Tipo não especificado")
		}
	}

	thread::sleep(Duration::from_secs(1));
    // h.join();
	shutdown();

}

fn normal_server(lister: Arc<TcpListener>, running: Arc<AtomicBool>){
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


#[warn(unused_variables)]
fn live_server(lister: Arc<TcpListener>, running: Arc<AtomicBool>, porta: u32){
    let (tx, rx) = channel::<u8>();

    let r = Arc::clone(&running);
    let p = thread::spawn(move || {
        // let mut websocket = TcpListener::bind(format!("0.0.0.0:{}", porta + 1)).unwrap();
        // let mut web_ = WebSocket::from_raw_socket(websocket, Role::Server, None);
        loop {
            let _ = rx.recv();
            println!("Ping");
            if !r.load(SeqCst) {
                break;
            }
        }
        
    });
    // vai ter o nome do arquivo(desconsiderar pastas por enquanto) e o tempo de ultima modificação
    let mut set: HashMap<String, SystemTime> = HashMap::new();
    // let mut i = 0;
    // não vai ser multi threading, no momento ainda estou vendo certinho como vai funcionar
    loop {
        if !running.load(SeqCst){
            break;
        }
        match lister.accept(){
            Ok((mut s, _)) => {
                soc_con(&mut s, &mut set); 
                s.shutdown(Shutdown::Both).unwrap();
                // TODO: Fazer a logica do running, sair com o ctrl
                // fazer um jeito de mandar o js para modificar as coisas
                // let lister_web = TcpListener::bind(format!("0.0.0.0:8001")).unwrap();
                // // fazer logica para modificar se for pasta
                // let file = File::open(format!("{}{}", FILE_SOURCE_PATH, "index.html")).unwrap();

                // let mut last = file.metadata().unwrap().modified().unwrap();
                // let mut b = false;
                // if !running.load(SeqCst) { // melhorar sainda
                //     break;
                // }
                // // apartir daqui ele trava
                // loop {
                //     if !b {
                //         let s2 = lister_web.accept().unwrap().0;
                //         let mut web_s = tungstenite::accept(s2).unwrap();
                //         let _ = web_s.send(tungstenite::Message::Text(String::new())).unwrap();
                //         // copiar de cima e fazer a logica para ele troicar de dados, ou seja sair desse loop e ir para o proximo req
                //         b = true;
                //     }
                //     thread::sleep(Duration::from_secs(1));
                //     // TODO: melhorar isso daqui
                //     let x = file.metadata().unwrap().modified().unwrap();
                //     if last != x {
                //         last = x;
                //         b = false;
                //     }
                // }

            }
            _ => {

            }
        }
        thread::sleep(Duration::from_millis(150)); // espera um pouvco
        for (nome, time_) in set.iter_mut(){
            let x = &format!("{}{}", FILE_SOURCE_PATH, nome);
            let p = Path::new(x);
            let c = p.metadata().unwrap().modified().unwrap();
            if c != *time_ {
                *time_ = c;
                // println!("foi");
                tx.send(1).unwrap();
            }
            // println!("{}{:?}", nome, time_.duration_since(SystemTime::UNIX_EPOCH).unwrap());
        }

    }
}
