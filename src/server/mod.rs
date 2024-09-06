use core::fmt;
use std::fs::ReadDir;
use std::io::{prelude::*, BufReader};
use std::task::Context;
use std::{fs, usize};
use std::net::TcpStream;
use std::path::{Path};
use build_html::{Html, HtmlContainer, HtmlPage};

mod log;
use log::*;

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
    // TODO: Ajeitar quando for nome de dir, dando algum erro
    let extension = Path::new(file_name).extension().unwrap().to_str().unwrap_or_else(|| "");
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
        _ => DIR
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
    fn new(stream: &mut TcpStream) -> Result<Request, String>{
        let request = read_req(stream);
        if request.len() < 2{
            return Err(String::from("Erro ao ler o request"));
        }
        // TODO: Erro de index out of the bound aqui(não é sempre que acontece, ele fala de erro no index 0)
        let uri: Vec<&str> = request[0].split(" ").collect(); // !
        let Host = request[1].split(" ").collect::<Vec<_>>()[1].to_string(); // ! aqui
        let r = Request {method: httpM_fromStr(&uri[0]), Host, required: uri[1][1..].to_string()}; // !
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
        let p = i.unwrap().path();
        if p.is_file(){
            let d = p.file_name().unwrap().to_str().unwrap();
            let t = getFile_type(d);
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
                good_response_make(&f, status, content.len()), content
            );
            stream.write(response.as_bytes()).unwrap();
        } else {
            // mensagem de erro que não achou um html(diferente de arquivo nao existe)
            status = 404;
            warning("Nenhum Html foi achado");
            let (header, html) = dir_html("", status);
            stream.write(format!("{}{}", header, html).as_bytes()).unwrap();
            // TODO: Ai vai fazer mostrar o html da pagina normal
        }
        // refatorar daqui para baixo
    } else if p.is_file() { 
        // mensagem de erro para caso não ache o arquivo em si(ta bugando tudo)
        // talvez usar o Path, ele retorna um erro caso o arquivo não exista
        match getFile_type(file_name) {
            TXT | HTML | CSS | JS | JSON => {
                let content = read_file_text(file_name).unwrap(); // talvez um panic aqui
                let response = format!(
                    "{}{}",
                    good_response_make(file_name, status, content.len()), content
                );
                stream.write(response.as_bytes()).unwrap();
            },
            _ => {
                let content = read_file_bytes(file_name).unwrap();
                let response = good_response_make(file_name, status, content.len());
                stream.write(response.as_bytes()).unwrap();
                stream.write(&content).unwrap();
            }
        }

    } else if p.is_dir(){
        // fazer response da pagina que seleciona um arquivo
        warning("Pasta requisitada");
        // TODO: mostrar o html da pagina que foi requisitada
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

// ExactSizeIterator é para usa o métood len
// T: Sized + ExactSizeIterator
pub fn good_response_make(file_send: &str, status_code: u32, content_len: usize) -> String {
    let status_ = header_make(status_code);

    let response = format!(
        "{}\r\nContent-Type: {}Content-Length: {}\r\n\r\n",
        status_, cont_type(file_send), content_len
    ); // depois desse texto é só colocar o content caso for de texte e caso não só manda os bytes dps
    response
}
/////////////////////////////////////////////////
// HTTP/1.1 404 Not Found
// Content-Type: text/html; charset=UTF-8
// Content-Length: 123

// <html>
// <head>
//     <title>404 Not Found</title>
// </head>
// <body>
//     <h1>404 Not Found</h1>
//     <p>The requested resource could not be found on this server.</p>
// </body>
// </html>
///////////////////////////////////
fn bad_response_make(status_code: u32) -> (String, String){
    // usando a crate build_html só para facilitar
    let status_ = header_make(status_code);
    let code_s = code_to_status(status_code);
    let html_error = HtmlPage::new()
        .with_title(&code_s)
        .with_header(1, &code_s)
        .with_paragraph("The requested resource could not be found on this server.");
    let Shtml = html_error.to_html_string();
    (format!(
        "{}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
        status_, Shtml.len()
    ), Shtml)
}

fn dir_html(pasta: &str, status_code: u32) -> (String, String){
    let status_ = header_make(status_code);
    let code_s = code_to_status(status_code);
    let mut html_dir = HtmlPage::new();
    let mut absolute = String::from(FILE_SOURCE_PATH);
    absolute.push_str(pasta);
    // println!("{}", absolute);
    let dir = fs::read_dir(absolute).unwrap();
    // compor o html aqui(for pelos itens)

    for i in dir {
        let p = i.unwrap().path();
        // html_error.add_link()
        // if p.is_file(){
        //     let d = p.file_name().unwrap().to_str().unwrap();
        //     let t = getFile_type(d);
        //     if d == "index.html"{
        //         return String::from("index.html");
        //     } else if t == HTML{
        //         tmp = d.to_string();
        //     }
        // }
        let mut Pname = String::from("/");
        // TODO: ajeitar isso
        Pname.push_str(pasta);
        // Pname.push_str("/");
        // mudar logica, pois o "não achar html", só funciona se isso tiver comentado
        // abrir outras funciona com isso, abrir arquivos dentro de pastas não gunciona
        let arq = p.file_name().unwrap().to_str().unwrap();
        Pname.push_str(arq);
        // println!("{:?}", Pname);
        html_dir.add_link(
            Pname, arq 
        ); 
        html_dir.add_paragraph("");
    }


    let Shtml = html_dir.to_html_string();
    (format!(
        "{}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
        status_, Shtml.len()
    ), Shtml)

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
    match Request::new(stream){
        Ok(req) => {
            print_rq(&req);
            match req.method {
                GET => {
                    // Caso o arquivo não exista, usar alguma forma de mandar o erro 404, tirar o status code de parametro e usar ele para ser decidido dentro da função
                    let _ = file_sender(stream, &req.required);
                },
                POST =>{
                    // implementar para mostrar os dados na tela, basicamente(colocar os dados no ENUM de post) 
                },
            };
        },
        Err(s) => {
            warning(&s);
        },
    };
}
