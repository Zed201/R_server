# R_server
Um servidor http criado a apartir do ultimo projeto do livro oficial de rust, basicamente servindo como um http.server módulo do python.

### TODO:
- [X] Verificar possível memory leak do fato de ter várias threads(não consegui só com o valgrind, pois ele da resultado errado, muito exagerado então imagino que esteja errado)(Ainda fica com um memory leak de 3.87k independente de quantas requisições sejam feitas)
- [ ] Aprimorar para algo como live server usando WebSocket(estruturar esquema no miro)
- [ ] Comentar melhor o server/mod.rs
- [X] Modificar como está a estrutura de Requets(talvez trocar para um type apenas de Hashmap)
- [X] Melhorar o threadpool, ou adicionar o tokio, ele não ta pegando no pc de casa(TOP PROBLEMAS MISTERIOSOS QUE N SEI, PQ PEGA AO RODAR COM O HEAPSTACK; POR ENQUANTO ACEITEI QUE ELE PEGA DO JEITO QUE TÀ)



////////////////////////////////////////////////////////////////////////////////////////////////
///// use std::net::TcpListener;
// /////////////use std::thread::spawn;
// use tungstenite::accept;

// /// A WebSocket echo server
// fn main () {
// 	// exemplo de criar websocket, mas precisa partir do navegador
//     let server = TcpListener::bind("127.0.0.1:9001").unwrap();
//     for stream in server.incoming() {
//         spawn (move || {
//             let mut websocket = accept(stream.unwrap()).unwrap();
//             loop {
//                 let msg = websocket.read().unwrap();
// 				println!("{:?}", msg);
//                 // We do not want to send back ping/pong messages.
//                 if msg.is_binary() || msg.is_text() {
//                     websocket.send(msg).unwrap();
//                 }
//             }
//         });
//     }
}
////////////////////////////////////////////////////////////////////////////////////////////////