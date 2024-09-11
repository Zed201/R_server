use super::Request;

use colored::Colorize;
use chrono::Local;

// * info genérico
pub fn info(s: &str){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[INFO {}]", n.format("%H:%M:%S")).green(),
        s
    );
}

// * só printando a estrutura de requisição
// TODO: Mudar quando mudar como a 'Resquest' vai funcionar
pub fn print_rq(r: &Request){
    info(format!("{}", r).as_str());
}

// * warning genérico
pub fn warning(s: &str){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[WARNING {}]", n.format("%H:%M:%S")).red(),
        s
    );
}

// * mensagem ao não achar um arquivo
pub fn file_not(s: &str){
    let s = s.white();
    warning(&format!("Arquivo {} não achado", s).red().to_string());
}

// * mensagem para desligar
pub fn shutdown(){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[WARNING {}]", n.format("%H:%M:%S")).red(),
        "DESLIGANDO".bright_red()
    );
}

pub fn on(){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[INFO {}]", n.format("%H:%M:%S")).green(),
        "LIGANDO".yellow()
    );
}