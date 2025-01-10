pub mod locallog;
use log::{debug, error, info, warn};

use std::convert::Infallible;

use tokio::net::TcpStream;

use hyper_util::{
	rt::{TokioExecutor, TokioIo},
	server::conn::auto,
};

use hyper::{body::Bytes, service::service_fn, Request, Response};

use std::path::{Path, PathBuf};

pub async fn process_web(stream: TcpStream) {
	let io = TokioIo::new(stream);
	if let Err(e) = auto::Builder::new(TokioExecutor::new())
		.serve_connection(io, service_fn(normal_web_server))
		.await
	{
		error!("{e}");
	}
}

// TODO: Em pastas ele não mostra o erro de notfound
async fn normal_web_server(r: Request<hyper::body::Incoming>) -> Result<Response<reqwest::Body>, Infallible> {
	let p = req_uri(r).await;
	if p.metadata().is_ok() {
		debug!("Arquivo existe");
		return Ok(send_file(&p).await);
	}
	Ok(not_found(&p))
}

use tokio::sync::broadcast::Receiver;

pub async fn process_live(stream: TcpStream, rx: Receiver<u8>) {
	let io = TokioIo::new(stream);
	if let Err(e) = auto::Builder::new(TokioExecutor::new())
		.serve_connection(
			io,
			service_fn(move |req| {
				// gambiarra por causa do borrowchekcer
				let mut rx = rx.resubscribe();
				async move { reload_server(req, rx).await }
			}),
		)
		.await
	{
		error!("{e}");
	}
}

// use reqwest::Body;

async fn reload_server(
	r: Request<hyper::body::Incoming>,
	mut rx: Receiver<u8>,
) -> Result<Response<reqwest::Body>, Infallible> {
	// testar com testo pelo send refresh, pois ele ta empilhando os dados, pode tamb├®m mandar
	// um html de reload
	//  unfold: https://docs.rs/futures-util-preview/latest/futures_util/stream/fn.unfold.html
	// body https://docs.rs/reqwest/latest/reqwest/struct.Body.html
	// notify  https://docs.rs/notify/latest/notify/

	// usar isso para dar append em alguma junto dos dados
	//
	// Redirecionar apenas os html para aqui, os outros source ele retorna com ser normal
	//	// mas adiciona ao hashmap de mudan├ºas, ai basicamente ele da hot reload quando ele
	//	notifica alguma mudan├ºa
	// toda essa pataquada é por causa do borred
	let p = req_uri(r).await;
	let p2 = p.clone();
	// se ele n├úo for html ele responde normal
	// TODO: Implementar o outo reload para outros alem de html
	// TODO: Dar um jeito de modularizar esse codigo com o do ser
	if p.metadata().is_ok() {
		if let Some(ex) = p.extension() {
			if ex != "html" && (p.is_file() || p.is_dir()) {
				// TODO: Melhorar essa condi├º├úo
				return Ok(send_file(&p).await);
			}
			// se for html ele manda um stream para fazer o auto reload
			let stream = async_stream::stream! {
			    let file_initial = read_fileb(p.as_path()).await;
			    if let Ok(v) = String::from_utf8(file_initial.to_vec()) {
				yield Ok::<_,Infallible>(v);
				debug!("Html file send");
			    }
			    // TODO:Trocar isso para o trigger da mudan├ºa dos arquivos
			    // Usar o notify como watcher para arquivos, ver alternativa de apenas
			    // analisar todos os arquivos ou só os requisitados, ambos imagino ter
			    // que trazer alguma estrutura compartilhada para essa funcao, no caso
			    // o channel para receber, que não deve ser oneshot
			    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

			    // debug!("Entrando no recive");
			 //    match rx.recv() {
				// Ok(_) => {},
				// Err(e) => debug!("Error reciver {e}"),
			 //    }
			    // let a = rx.try_recv(); // nao recebe de jeito nenhum
			    // debug!("saiu do recive {:?}", a);

			    // o resultado no recive é minimamenteo aceitavel pois ele fica dando
			    // reload direto, mas tentar resolver, que o problema é apenas o
			    // channel

			    let update_chunk = String::from("<script>window.location.reload();</script>");
			    debug!("Reload code send");
			    info!("Reload");
			    yield Ok::<_, Infallible>(update_chunk);

			};
			let b = reqwest::Body::wrap_stream(stream);
			if let Ok(r) = Response::builder()
				.header("Content-Type", "text/html")
				.header("Cache-Control", "no-cache")
				.header("Connection", "keep-alive")
				.body(b)
			{
				return Ok(r);
			}
		}
	}
	Ok(not_found(&p2))
}

async fn send_file(p: &PathBuf) -> Response<reqwest::Body> {
	debug!("Path: {:?} - send_file", p);
	if p.is_file() {
		info!("Request file {:?}", p);
		let v = read_fileb(p).await;
		return response_builder(v);
	} else if p.is_dir() {
		// deve ter is_dir() pois ele pode acabar entrando aqui se o
		// try_exist falhar por alguma falta de permiss├úo
		// criar o html do diretorio
		//mesmo tirandoo try_exist e colocando o metadate deixei desse jeito aqui
		if let Ok(page_dir) = read_dirb(p).await {
			info!("Directory '{:?}' requested", p);
			return response_builder(page_dir);
		}
	}
	warn!("Blank file send");
	Response::default()
}

fn not_found(p: &PathBuf) -> Response<reqwest::Body> {
	warn!("Path {:?} not founded", p);
	// retornar msg de erro
	let error_page: maud::Markup = html! {
	    h1 { "Resource " (format!("{:?}", p)) " not Found" }
	};
	response_builder(Bytes::from(error_page.into_string()))
}

async fn req_uri(r: Request<hyper::body::Incoming>) -> PathBuf {
	let uri = r.uri();
	debug!("Uri '{:?}' - req_uri", uri);
	if uri == "/" {
		let mut tmp_p = PathBuf::default();
		// pesquisar o index.html ou qualquer html se não achar o index dentro do /
		if let Ok(mut dir) = read_dir("./").await {
			while let Ok(Some(entry)) = dir.next_entry().await {
				let p = entry.path();
				if let Some(n) = p.file_name() {
					if n == "index.html" {
						debug!("Found index.html");
						return p;
					}
				}
				if let Some(e) = p.extension() {
					if e == "html" {
						tmp_p = p;
					}
				}
			}
			// se ele nao achar html ele vai retornar um um resource not found
			debug!("Returning other html");
			return tmp_p;
		}
	}
	let p = PathBuf::from(uri.path().trim_start_matches('/'));
	debug!("Path {:?} - req_uri", p);
	p
}

// criador modularizado
fn response_builder(b: Bytes) -> Response<reqwest::Body> {
	Response::new(reqwest::Body::from(b))
}

use maud::{html, PreEscaped};
use tokio::fs::{read, read_dir};
async fn read_fileb(name: &Path) -> Bytes {
	if let Ok(v) = read(name).await {
		return Bytes::from(v);
	}
	Bytes::new()
}

async fn read_dirb(name: &Path) -> std::io::Result<Bytes> {
	if let Ok(mut dir) = read_dir(name).await {
		// nao e um iterator como a versao do std entao modifica
		let h = html! {
		    h1 {"Directory '" (name.display()) "'"}
			@while let Some(entry) = dir.next_entry().await? {
			@let pa = entry.path().display().to_string(); // pega o path absoluto
			@let link = PreEscaped(format!("/{}", pa)); // adiciona um / no começo para
								    // o link ser absoluto
			@let name = PreEscaped(pa); // Nome do arquivo normal sem " "
			a href={(link)} {(name)}
			br;
		    }
		};
		return Ok(Bytes::from(h.into_string()));
	}
	Err(std::io::Error::new(
		std::io::ErrorKind::InvalidData,
		"Cannot read the directory",
	))
}
