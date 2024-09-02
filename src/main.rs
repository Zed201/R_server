use std::{net::TcpListener, net::TcpStream};
use std::io::{prelude::*, BufReader};

fn read_req(mut request: TcpStream){
    let buffer =  BufReader::new(&mut request);
    let request: Vec<_> = buffer.
        lines().map(|result| result.unwrap()).take_while(|linha| !linha.is_empty()).collect();
   println!("{request:?}");

}

fn main() {
    let listenert = TcpListener::bind("127.0.0.1:8080").unwrap();
    for s in listenert.incoming(){
        let stream = s.unwrap();

        read_req(stream)
    }
}
