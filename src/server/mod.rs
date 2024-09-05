use core::fmt;
use std::fs::ReadDir;
use std::io::{prelude::*, BufReader};
use std::{fs, usize};
use std::net::TcpStream;

static FILE_SOURCE_PATH: &str = "./test_source/";

enum httpMet{
    GET,
    POST,
}

// não so de file, mas o dir entra para compor o tipo de dado
#[derive(PartialEq)]
enum File_type{
    TXT,
    HTML,
    CSS,
    JS,
    PNG,
    JPEG,
    JPG,
    JSON,
    PDF,
    ICO,
    DIR
}

use File_type::*;

fn getFile_type(file_name: &str) -> File_type{
    let extension = *file_name.split(".").collect::<Vec<_>>().last().unwrap();
    match extension {
        "txt" => TXT,
        "html" => HTML,
        "css" => CSS,
        "js" => JS,
        "png" => PNG,
        "jpeg" => JPEG,
        "jpg" => JPG,
        "json" => JSON,
        "pdf" => PDF,
        "ico" => ICO,
        _ => TXT
    }
}

use httpMet::*;

impl fmt::Display for httpMet{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            GET => write!(f, "GET"),
            POST => write!(f, "POST"),
        }
    }
}

fn httpM_toStr(method: httpMet) -> String{
    match method {
        GET => String::from("GET"),
        POST => String::from("POST"),
    }
}

fn httpM_fromStr(method: &str) -> httpMet{
    match method {
        "GET" => GET,
        "POST" => POST,
        _ => GET
    }
}

struct Request{
    method: httpMet,
    Host: String,
    required: String, // apenas nome do arquico que foi pedido, sem / no final, se for vazio é pq ele ta pedindo o index.html
}

impl Request{
    fn new(stream: &mut TcpStream) -> Request{
        let request = read_req(stream);
        let uri: Vec<&str> = request[0].split(" ").collect();
        let Host = request[1].split(" ").collect::<Vec<_>>()[1].to_string();
        Request {method: httpM_fromStr(&uri[0]), Host, required: uri[1][1..].to_string()}
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} request from {} in {}", self.method, self.Host, self.required)
    }
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

// procurar o aquivo index caso o request seja /, caso não encontre o index.html, retornar um html qualquer(o ultimo na iteração)
// caso não tenha html ele retorna vazio, aí envia error 404
fn search_index() -> String{
    let dir = fs::read_dir(FILE_SOURCE_PATH).unwrap();
    let mut tmp: String = String::new();
    for i in dir {
        let d = i.unwrap().file_name();
        let t = getFile_type(d.to_str().unwrap());
        if d == "index.html"{
            return String::from("index.html");
        } else if t == HTML{
            tmp = d.to_str().unwrap().to_string();
        }
    }
    tmp
}

pub fn file_sender(stream: &mut TcpStream, status: u32, file_name: &str){
    // caso seja de texto coloca no final do responde
    // TODO: tratar paca casos de / procuar o index.html, caso não ache ele ou outro aquivo, mandar um 404 error
    // TODO: Fazer ele primeiro procuar pelo html, fazer algum if com o dado do nome para ele verificar ser html e mandar o resto assim
    // TODO: Depois pensar em algo para ele enviar a pagina de ver os arquivos e diretorios(fazer uma forma de representar os diretorios, para ser representado de forma recursiva depois)

    // se o nome de arquivo for "" vazio
    
    if file_name.len() == 0{
        let f = search_index();
        if f.len() > 0 {
            let content = read_file_text(&f).unwrap();
            let response = format!(
                "{}{}",
                response_make(&f, status, content.len()), content
            );
            stream.write(response.as_bytes()).unwrap();
        } else {
            // mensagem de erro
        }
        // refatorar daqui para baixo
    } else { 
        match getFile_type(file_name) {
            TXT | HTML => {
                let content = read_file_text(file_name).unwrap();
                let response = format!(
                    "{}{}",
                    response_make(file_name, status, content.len()), content
                );
                stream.write(response.as_bytes()).unwrap();
            },
            _ => {
                let content = read_file_bytes(file_name).unwrap();
                let response = format!(
                    "{}",
                    response_make(file_name, status, content.len())
                );
                stream.write(response.as_bytes()).unwrap();
                stream.write(&content).unwrap();
            }
        }
    }
    // caso seja uma imagem ou coisa parecida
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
    let extension =  getFile_type(file_name);
    match extension {
        TXT => "text/plain\r\n",
        HTML => "text/html\r\n",
        CSS => "text/css\r\n",
        JS => " text/javascript\r\n",
        PNG | JPEG | JPG => "image/png\r\n", // completar com os outros tipos
        JSON => "application/json\r\n",
        PDF => "application/pdf\r\n",
        ICO => "image/x-icon\r\n",
        _ => "text/plain\r\n", // modficar para ser o de dir
    }
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
    let req = Request::new(stream);
    println!("{}", req);
    match req.method {
        GET => {
            // Caso o arquivo não exista, usar alguma forma de mandar o erro 404, tirar o status code de parametro e usar ele para ser decidido dentro da função
            let _ = file_sender(stream, 200, &req.required);
        },
        POST =>{
            // implementar para mostrar os dados na tela, basicamente(colocar os dados no ENUM de post) 
        },
    };
}
