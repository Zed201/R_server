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
use chrono::Local;
use log::{self, info, warn, LevelFilter};
use once_cell::sync::Lazy;
use std::env;
use std::str::FromStr;
// RUST_LOG=[target][=][level][,...]
// Levels(crescente): error, warn, info, debug, trace, off

struct Logger;
// Implementar a questão de cores no logger

impl log::Log for Logger {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		metadata.level() <= *MAX_LEVEL
	}
	fn flush(&self) {}
	fn log(&self, record: &log::Record) {
		if self.enabled(record.metadata()) {
			let n = Local::now();
			println!("[{} - {}]: {}", record.level(), n.format("%H:%M:%S"), record.args());
		}
	}
}

static L: Logger = Logger;
// Usa a inicialização lazy para pegar o Maximo de Log
static MAX_LEVEL: Lazy<LevelFilter> = Lazy::new(|| match env::var("LOG") {
	Ok(val) => match LevelFilter::from_str(val.as_str()) {
		Ok(v) => v,
		Err(_) => LEVEL_DEFAULT,
	},
	Err(_) => LEVEL_DEFAULT,
});
static LEVEL_DEFAULT: LevelFilter = LevelFilter::Info;

// inicia o logger principal
pub fn init_logger() {
	let _ = log::set_logger(&L);
	log::set_max_level(*MAX_LEVEL);
}

pub fn on(port: u16) {
	info!("Ligando o servidor na porta {port}");
}

pub fn off() {
	warn!("Desligando o servidor")
}
