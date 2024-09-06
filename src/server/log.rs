use super::{Request};

use colored::{Colorize, Color::*, Color, ColoredString};
use chrono::Local;

pub fn info(s: &str){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[INFO {}]", n.format("%H:%M:%S")).green(),
        s
    );
}

pub fn print_rq(r: &Request){
   info(format!("{}", r).as_str());
}

pub fn warning(s: &str){
    let n = Local::now();
    println!(
        "{}: {}",
        format!("[WARNING {}]", n.format("%H:%M:%S")).red(),
        s
    );
}

pub fn file_not(s: &str){
    let s = s.white();
    warning(&format!("Arquivo {} não achado", s).red().to_string());
}
