use std::io::{prelude::*, BufReader};
use std::time::Duration;
use std::{fs, thread};
use std::net::TcpStream;

pub fn read_req(stream: &mut TcpStream) -> Vec<String> {
    let buffer = BufReader::new(stream);
    let request: Vec<String> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|linha| !linha.is_empty())
        .collect();
    request
}

pub fn code_to_status(code: u32) -> String {
    match code {
        200 => String::from("200 OK"),
        404 => String::from("404 NOT FOUND"),
        _ => String::from("200 Ok"),
    }
}

// fn cont_type(file_name: &str) -> {

// }

///////////////////////////////////////
/// A estrutura básica da resposta do http é 
/// HTTP/1.1 (code_status) (code_msg)
/// Content-Type: (Tipo de conteudo);
/// <body> EM BYTES 
///////////////////////////////////////
/// (Tipo de conteudo)(final com \r\n)
/// text/javascript
/// text/css
/// text/html
/// text/plain
/// image/x-icon
/// image/png // implementar apenas png
/// application/json
/// application/pdf

pub fn file_sender(stream: &mut TcpStream, status: u32, file: &str) -> Result<(), String> {
    let status_line = format!("HTTP/1.1 {}", code_to_status(status));
    if file.ends_with(".html") {
        let content = match fs::read_to_string(format!("src/{}", file)) {
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que é texto", file)),
        };

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\n\r\n{content}",
            content.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let content = match fs::read(format!("src{}", file)) {
            Ok(l) => l,
            Err(_) => return Err(format!("Erro ao pegar arquivo {} que não é texto", file)),
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/x-icon\r\nContent-Length: {}\r\n\r\n",
            content.len()
        );
        stream.write(response.as_bytes()).unwrap();
        stream.write(&content).unwrap();
    }
    Ok(())
}

pub fn handle_con(stream: &mut TcpStream) {
    let request = read_req(stream);
    let uri: Vec<&str> = request[0].split(" ").collect();

    // melhorar aqui, para redirecionar para um padrão de html
    if uri[0] == "GET" {
        println!("{:?}", uri);
        let file = if uri[1] != "/" { uri[1] } else { "index.html" };
        let _ = file_sender(stream, 200, file).unwrap();
    }
    thread::sleep(Duration::from_secs(5));
}