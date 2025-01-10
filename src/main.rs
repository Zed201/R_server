mod server;
use clap::*;
use log::{debug, error, info, warn};
use notify::{Config, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use server::locallog::*;
use server::{process_live, process_web};
use std::path::Path;
use std::process::exit;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
// use tokio::time::{sleep, Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Mode {
	Web,
	Live,
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
			    match m {
				Mode::Web => {
				    loop {
					if let Ok((s, _)) = listener.accept().await{
					    debug!("New request accepted");
					    tokio::spawn(async move {
						process_web(s).await
					    });
					} else {
					    warn!("Connection not accepted");
					}
				    }
				},
				Mode::Live => {

				let path = Path::new(".");

				    let (tx, rx) = std::sync::mpsc::channel();
				    // tente fazer sem os unwarp, mas por algum motivo nao vai com if let,
				    // tentei ate em outro arquivo/projeto, provavelmente por incompetencia
				    // minha mas fazer oq
				    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
				    watcher.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();

				    let (ttx, _) = tokio::sync::broadcast::channel(100); // se for
				    // trocar para mpsc deve trocar as implementações para arc
				    let ttx2 = ttx.clone();
				    tokio::spawn(async move {
					while let Ok(i) = rx.recv() { // vai servir como broadcast manual pois o notify
					    // não aceita o tokio::broadcast
					    if let Ok(r) = i {
						if matches!(r.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)){
						    debug!("modificou");
						    match ttx2.send(1) { // nao ta envidando por
							    // algum motivo
							    Ok(o) => debug!("send - {o}"),
							    Err(e) => debug!("send erro - {e}"),
						    }
						}
					    }
				    }});
				    loop {
					if let Ok((s, _)) = listener.accept().await{
					    debug!("New request accepted");
					    let trx2 = ttx.subscribe();
					    tokio::spawn(async move {
						process_live(s, trx2).await
					    });
					} else {
					    warn!("Connection not accepted");
					}

				    }
				},
			    }
		    } => {}
		}
	} else {
		error!("Erro ao bindar a porta especificada");
		exit(1);
	}
}

/// aaaa
use notify::event::EventKind;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender};

// cli args
fn commads() -> (u16, Mode) {
	// TODO: retrabalhar essas strings
	let port_str: String = format!("Escolha da porta na qual o servidor vai ficar ouvindo, \nse for escolhido o modo Live, o servidor vai ficar na 'porta' \n e o websocket vai ficar na {}\n", 1);
	let mode_str: &str =
		"Escolha entre os modos web(servidor http normal) e o \nLive(servidor funcionando como live server)\n";
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
		.unwrap_or(Mode::Web);
	(_porta, _mode)
}
