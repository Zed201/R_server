mod server;
use clap::*;
use log::{debug, error, warn};
use server::locallog::*;
use server::process_web;
use std::process::exit;
use tokio::net::TcpListener;
// use tokio::time::{sleep, Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Mode {
	WEB,
	LIVE,
}

#[tokio::main]
async fn main() {
	let (p, m) = commads();
	debug!("Porta {p} e modo {:?}", m);
	init_logger();
	on(p);

	// codigo de timer pronto para testar algo
	// let timer = tokio::spawn(async {
	// 	sleep(Duration::from_secs(2)).await;
	// });

	if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{p}")).await {
		tokio::select! {
			_ = async {
				if tokio::signal::ctrl_c().await.is_ok() {
					off();
				}
			} => {}, // get out from ctrl+C
			// _ = timer => {},
			_ = async {
				loop {
					if let Ok((s, _)) = listener.accept().await{
						debug!("Nova requisição aceita");
						tokio::spawn(async move {
							 process_web(s).await
						});
					} else {
						warn!("Conexão não conseguiu ser aceita");
					}
				}
			} => {}
		}
		// TODO: Fazer parte do live com tungstenite?
	} else {
		error!("Erro ao bindar a porta especificada");
		exit(1);
	}
}

// cli args
fn commads() -> (u16, Mode) {
	// TODO: retrabalhar essas strings
	let port_str: String = format!("Escolha da porta na qual o servidor vai ficar ouvindo, \nse for escolhido o modo live, o servidor vai ficar na 'porta' \n e o websocket vai ficar na {}\n", 1);
	let mode_str: &str =
		"Escolha entre os modos web(servidor http normal) e o \nlive(servidor funcionando como live server)\n";
	let cmd = Command::new("R_server")
		.args(&[
			Arg::new("port_no_flag")
				.help(port_str.clone())
				.required(false)
				.value_parser(clap::value_parser!(u16))
				.index(1),
			Arg::new("port")
				.short('p')
				.long("port")
				.help(port_str)
				.required(false)
				.value_parser(clap::value_parser!(u16)),
			Arg::new("mode_no_flag")
				.help(mode_str)
				.required(false)
				.value_parser(clap::builder::EnumValueParser::<Mode>::new())
				.index(2),
			Arg::new("mode")
				.short('m')
				.long("mode")
				.help(mode_str)
				.required(false)
				.value_parser(clap::builder::EnumValueParser::<Mode>::new()),
		])
		.get_matches();
	let _porta = cmd
		.get_one::<u16>("port")
		.or_else(|| cmd.get_one::<u16>("port_no_flag"))
		.cloned()
		.unwrap_or(8000);
	let _mode = cmd
		.get_one::<Mode>("mode")
		.or_else(|| cmd.get_one::<Mode>("mode_no_flag"))
		.cloned()
		.unwrap_or(Mode::WEB);
	(_porta, _mode)
}
