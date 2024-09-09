use core::fmt;
use std::io::{prelude::*, BufReader};
use std::os::linux::raw::stat;
use std::{fs, usize};
use std::net::TcpStream;
use std::path::Path;
use std::collections::HashMap;

use build_html::{Html, HtmlContainer, HtmlPage};

pub mod log;
use log::*;

static FILE_SOURCE_PATH: &str = "./test_source/";
enum HttpMet{
    GET,
    POST,
}

use HttpMet::*;

fn http_mfrom_str(method: &str) -> HttpMet{
    match method {
        "GET" => GET,
        "POST" => POST,
        _ => GET
    }
}

impl fmt::Display for HttpMet{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            GET => write!(f, "GET"),
            POST => write!(f, "POST"),
        }
    }
}

pub fn code_to_status(code: u32) -> String {
    match code {
        200 => String::from("200 OK"),
        404 => String::from("404 NOT FOUND"),
        _ => String::from("200 Ok"),
    }
}

#[derive(PartialEq)]
enum FileType{
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
    DIR,
}

use FileType::*;

fn get_file_type(file_name: &str) -> FileType{
    match Path::new(file_name).extension(){
        Some(extension) =>{
            match extension.to_str().unwrap() {
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
                _ => DIR
            }

        },
        None =>{
            if file_name.contains("."){
                // arquivo hidden
                return TXT;
            } else {
                return DIR;
            }

        }
    }

}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TXT => write!(f, "text/plain\r\n"),
            HTML => write!(f, "text/html\r\n"),
            CSS => write!(f, "text/css\r\n"),
            JS => write!(f, " text/javascript\r\n"),
            PNG | JPEG | JPG => write!(f, "image/png\r\n"), 
            JSON => write!(f, "application/json\r\n"),
            PDF => write!(f, "application/pdf\r\n"),
            ICO => write!(f, "image/x-icon\r\n"),
            _ => write!(f, "text/html\r\n"),
        }
    }
}

pub struct Request{
    method: HttpMet,
    required: String, // apenas nome do arquico que foi pedido, sem / no final, se for vazio é pq ele ta pedindo o index.html
}

impl Request{
    // TODO: Ajustar para ler pegar melhor a req, usar melhor o request(se for precisar ler mais coisa da requisição), 
    // talvez usar um type pois ele seria só um HashMap<String, String>, ou modificar apenas a
    // struct, mas aí ia ter de modificar todo o resto, onde ele é acessado
    fn new(stream: &mut TcpStream) -> Result<Request, String>{
        let request = read_req(stream);
        let r = String::from("GET");
        let r = Request {method: http_mfrom_str(request.get("User-Agent").unwrap_or_else(|| &r)), required: request["required"].clone()};
        Ok(r)
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.required.len() > 0{
            write!(f, "{} request for {}", self.method, self.required)
        } else {
            write!(f, "{} request for /", self.method)
        }
    }
}

pub fn read_req(stream: &mut TcpStream) -> HashMap<String, String> {
    let mut buffer = BufReader::new(stream);
    let mut f = String::new();
    let _ = buffer.read_line(&mut f).expect("Erro ao ler o método da requisição");
    let mut mapa: HashMap<String, String> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|linha| !linha.is_empty())
        .map(|linha| {
            let mut s = linha.split(": ");
            if let (Some(key), Some(value)) = (s.next(), s.next()) {
                return (key.to_string(), value.to_string())
            }
            (String::new(), String::new()) 
        })
        .collect();
    let uri: Vec<&str> = f.split(" ").collect();
    // tratado de forma meio porca mais 
    mapa.insert("method".to_string(), uri.get(0).unwrap_or_else(|| &" ").to_string());
    mapa.insert("required".to_string(), uri.get(1).unwrap_or_else(|| &" ")[1..].to_string());
    mapa
}


pub fn header_make(status_code: u32, cont_type: FileType, len: usize) -> String {
    format!("HTTP/1.1 {}\r\nContent-Type: {}Content-Length: {}\r\n\r\n", code_to_status(status_code), cont_type, len)
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
        let p = i.unwrap().path();
        if p.is_file(){
            let d = p.file_name().unwrap().to_str().unwrap();
            let t = get_file_type(d);
            if d == "index.html"{
                return String::from("index.html");
            } else if t == HTML{
                tmp = d.to_string();
            }
        }
    }
    tmp
}

// TODO? Talvez refatorar essa função, pois ta muitca coisa nela
pub fn file_sender(stream: &mut TcpStream, file_name: &str){
    // se o nome de arquivo for "" vazio
    let mut status = 200;
    let mut relative_p = String::from(FILE_SOURCE_PATH);
    relative_p.push_str(file_name);
    let p = Path::new(&relative_p);
    if file_name.len() == 0{
        let f = search_index();
        if f.len() > 0 {
            let content = read_file_text(&f).unwrap();
            let response = format!(
                "{}{}",
                header_make(status, get_file_type(file_name), content.len()), 
                content
            );
            stream.write(response.as_bytes()).unwrap();
        } else {
            // mensagem de erro que não achou um html(diferente de arquivo nao existe)
            status = 404;
            warning("Nenhum Html foi achado, mostrando pasta atual");
            let (header, html) = dir_html("", status);
            stream.write(format!("{}{}", header, html).as_bytes()).unwrap();
        }

    } else if p.is_file() { 
        match get_file_type(file_name) {
            TXT | HTML | CSS | JS | JSON => {
                let content = read_file_text(file_name).unwrap(); // talvez um panic aqui
                let response = format!(
                    "{}{}",
                    header_make(status, get_file_type(file_name), content.len()),
                    content
                );
                stream.write(response.as_bytes()).unwrap();
            },
            _ => {
                let content = read_file_bytes(file_name).unwrap();
                let header = header_make(status, get_file_type(file_name), content.len());
                stream.write(header.as_bytes()).unwrap();
                stream.write(&content).unwrap();
            }
        }

    } else if p.is_dir(){
        info(format!("Pasta {} requisitada", file_name).as_str());
        let(header, html) = dir_html(file_name, status);
        stream.write(format!("{}{}", header, html).as_bytes()).unwrap();

    } else {
        file_not(file_name);
        status = 404;
        let (header, html) = bad_response_make(status);
        stream.write(format!("{}{}", header, html).as_bytes()).unwrap();
    }
    // caso seja uma imagem ou coisa parecida
}

fn bad_response_make(status_code: u32) -> (String, String){
    // usando a crate build_html só para facilitar
    let code_s = code_to_status(status_code);
    let html_error = HtmlPage::new()
        .with_title(&code_s)
        .with_header(1, &code_s)
        .with_paragraph("The requested resource could not be found on this server.");
    let shtml = html_error.to_html_string();
    (header_make(status_code, HTML, shtml.len()), shtml)
}


fn dir_html(pasta: &str, status_code: u32) -> (String, String){
    let mut html_dir = HtmlPage::new();
    let mut absolute = String::from(FILE_SOURCE_PATH);
    absolute.push_str(pasta);
    let dir = fs::read_dir(absolute).unwrap();

    for i in dir {
        let p = i.unwrap().path();

        let mut pname = String::from("/");
        // TODO: melhorar isso, em questão de desempenho
        pname.push_str(pasta);
        if pasta.len() > 0{
            pname.push_str("/");
        }
        // mudar logica, pois o "não achar html", só funciona se isso tiver comentado
        // abrir outras funciona com isso, abrir arquivos dentro de pastas não gunciona
        let arq = p.file_name().unwrap().to_str().unwrap();
        pname.push_str(arq);
        html_dir.add_link(
            pname, arq 
        ); 
        html_dir.add_paragraph("");
    }


    let shtml = html_dir.to_html_string();
    (header_make(status_code, HTML, shtml.len()), shtml)

}

pub fn handle_con(stream: &mut TcpStream) {
    match Request::new(stream){
        Ok(req) => {
            print_rq(&req);
            match req.method {
                GET => {
                    // Caso o arquivo não exista, usar alguma forma de mandar o erro 404, tirar o status code de parametro e usar ele para ser decidido dentro da função
                    let _ = file_sender(stream, &req.required);

                },
                POST => {
                    // TODO: implementar para mostrar os dados no terminal, basicamente(colocar os dados no ENUM de post) 
                },
            };
            
        },
        Err(s) => {
            warning(&s);
        },
    };
}
