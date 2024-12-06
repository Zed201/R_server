// use super::Request;

// use chrono::Local;
// use colored::Colorize;

// // * info genérico
// pub fn info(s: &str) {
// 	let n = Local::now();
// 	println!("{}: {}", format!("[INFO {}]", n.format("%H:%M:%S")).green(), s);
// }

// // * só printando a estrutura de requisição
// // TODO: Mudar quando mudar como a 'Resquest' vai funcionar
// pub fn print_rq(r: &Request) {
// 	info(format!("{}", r).as_str());
// }

// // * warning genérico
// pub fn warning(s: &str) {
// 	let n = Local::now();
// 	println!("{}: {}", format!("[WARNING {}]", n.format("%H:%M:%S")).red(), s);
// }

// // * mensagem ao não achar um arquivo
// pub fn file_not(s: &str) {
// 	let s = s.white();
// 	warning(&format!("Arquivo {} não achado", s).red().to_string());
// }

// // * mensagem para desligar
// pub fn shutdown() {
// 	let n = Local::now();
// 	println!(
// 		"{}: {}",
// 		format!("[WARNING {}]", n.format("%H:%M:%S")).red(),
// 		"DESLIGANDO".bright_red()
// 	);
// }

// pub fn on() {
// 	let n = Local::now();
// 	println!(
// 		"{}: {}",
// 		format!("[INFO {}]", n.format("%H:%M:%S")).green(),
// 		"LIGANDO".yellow()
// 	);
// }

// pub fn reload(){
// 	let n = Local::now();
// 	println!(
// 		"{}: {}",
// 		format!("[INFO {}]", n.format("%H:%M:%S")).green(),
// 		"RELOAD".green()
// 	);
// }

//////////////////////////////////////////////////////////
use log::{self, Level, LevelFilter};
use std::io::Write;
use chrono::Local;
use std::env;
// RUST_LOG=[target][=][level][,...]
// Levels(crescente): error, warn, info, debug, trace, off

struct Logger;

impl log::Log for Logger {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		metadata.level() <= log::Level::Info
	}
	fn flush(&self) {
		
	}
	fn log(&self, record: &log::Record) {
		if self.enabled(record.metadata()){
			let n = Local::now();
			println!("[{} - {}]: {}", 
			record.level(), n.format("%H:%M:%S"), record.args());
		}
	}
}

static L: Logger = Logger;

pub fn init_logger() -> Result<(), log::SetLoggerError>{
	let logMax = match env::var("LOG"){
		Ok(val) => val,
		Err(_) => String::from("INFO")
	};
	// funcionando mais ou menos
	log::set_logger(&L).map(|()| 
		log::set_max_level(
			match logMax.to_uppercase().as_str() {
				"DEBUG" => LevelFilter::Debug,
				_ => LevelFilter::Warn
			}
		)	
	)
}