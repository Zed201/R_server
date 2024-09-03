use std::fmt::format;
use std::io::{prelude::*, BufReader};
use std::time::Duration;
use std::{fs, thread, usize};
use std::net::TcpStream;
use std::io;

static FILE_SOURCE_PATH: &str = "src/source";

enum httpMet{
    GET,
    POST,
}
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

pub fn header_make(status_code: u32) -> String {
    format!("HTTP/1.1 {}", code_to_status(status_code))
}

pub fn read_file_text(file_name: &str) -> Result<String, String>{   
    let content = match fs::read_to_string(format!("{}{}", FILE_SOURCE_PATH, file_name)) {
        Ok(l) => l,
        Err(_) => return Err(format!("Erro ao pegar arquivo {}{}", FILE_SOURCE_PATH, file_name)),
    };
    Ok(content)
}

pub fn read_file_bytes(file_name: &str) -> Result<Vec<u8>, String>{
    let content = match fs::read(format!("{}{}", FILE_SOURCE_PATH, file_name)) {
        Ok(l) => l,
        Err(_) => return Err(format!("Erro ao pegar arquivo {}{}", FILE_SOURCE_PATH, file_name)),
    };
    Ok(content)
}
// ExactSizeIterator é para usa o métood len
// T: Sized + ExactSizeIterator
pub fn response_make(file_send: &str, status_code: u32, content_len: usize) -> String {
    let status_ = header_make(status_code);

    let response = format!(
        "{}\r\nContent-Type: {}Content-Length: {}\r\n\r\n",
        status_, cont_type(file_send), content_len
    ); // depois desse texto é só colocar o content caso for de texte e caso não só manda os bytes dps
    response
}

// converter o data_type em Content-Type
pub fn cont_type<'a>(file_name: &'a str) -> &'a str {
    let extension =  file_name.split(".").collect::<Vec<_>>();
    match extension[1] {
        "txt" => "text/plain\r\n",
        "html" => "text/html\r\n",
        "css" => "text/css\r\n",
        "js" => " text/javascript\r\n",
        "png" | "jpeg" | "jpg" => "image/png\r\n", // completar com os outros tipos
        "json" => "application/json\r\n",
        "pdf" => "application/pdf\r\n",
        "ico" => "image/x-icon\r\n",
        _ => "text/plain\r\n"
    }
}

pub fn file_sender(stream: &mut TcpStream, status: u32, file_name: &str){
    // caso seja de texto coloca no final do responde
    if file_name.ends_with("html"){ // fazer um ir para casos de text, não so html
        let content = read_file_text(file_name).unwrap();
        let response = format!(
            "{}{}",
            response_make(file_name, status, content.len()), content
        );
        stream.write(response.as_bytes()).unwrap();
    } else {
        // ta dando algo errado ao mandar a imagem
        let content = read_file_bytes(file_name).unwrap();
        let response = format!(
            "{}",
            response_make(file_name, status, content.len())
        );
        stream.write(response.as_bytes()).unwrap();
        stream.write(&content).unwrap();
    }
    // caso seja uma imagem ou coisa parecida
}

///////////////////////////////////////
/// A estrutura básica da resposta do http é 
/// HTTP/1.1 (code_status) (code_msg)[X]
/// Content-Type: (Tipo de conteudo);
/// <body> EM BYTES ou não
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



pub fn handle_con(stream: &mut TcpStream) {
    let request = read_req(stream);
    let uri: Vec<&str> = request[0].split(" ").collect();

    // melhorar aqui, para redirecionar para um padrão de html
    if uri[0] == "GET" {
        println!("{:?}", uri);
        let file = if uri[1] != "/" { uri[1] } else { "/index.html" };
        // retirar o primeiro elemenot de 
        let _ = file_sender(stream, 200, file);
    }
}
