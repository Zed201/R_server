// // procurar o aquivo index caso o request seja /, caso não encontre o index.html, retornar um html qualquer(o ultimo na iteração)
// // caso não tenha html ele retorna vazio, aí envia error 404
// fn search_index() -> String {
//     if let Ok(dir) = fs::read_dir(FILE_SOURCE_PATH) {
//         let mut tmp: String = String::new();
//         for i in dir {
//             if let Ok(p) = i {
//                 let p = p.path();
//                 if p.is_file() {
//                     let d = p.file_name().unwrap().to_str().unwrap();
//                     let t = get_file_type(d);
//                     if d == "index.html" {
//                         return String::from("index.html");
//                     } else if t == HTML {
//                         tmp = d.to_string();
//                     }

//                 }
//             }
//         }
//         return tmp;
//     }
//     String::new()
// }

// pub fn soc_con(stream: &mut TcpStream, set: &mut HashMap<String, SystemTime>) {
//     match Request::new(stream) {
//         Ok(req) => {
//             print_rq(&req);
//             // não tem suporte para post e tal
//             let mut r = req.data["required"].clone();
//             if r.len() == 0 {
//                 // melhorar isso
//                 r = search_index();
//             }
//             // TODO: Melhorar esses métodos extensos
//             let _ = file_sender(stream, &r);
//             // tratar erros
//             //
//             if let Ok(l) = Path::new(&format!("{}{}", FILE_SOURCE_PATH, r)).metadata(){
//                 if let Ok(l) = l.modified() {
//                     set.insert(r, l);
//                 }
//             }
//         }
//         Err(s) => {
//             warning(&s);
//         }
//     }
// }

pub mod locallog;
use log::{debug, error, info, warn};

use std::convert::Infallible;

use tokio::net::TcpStream;

use hyper_util::{
	rt::{TokioExecutor, TokioIo},
	server::conn::auto,
};

use hyper::{
	body::{Body, Bytes},
	service::service_fn,
	Request, Response,
};

use http_body_util::Full;

use std::path::Path;

pub async fn process_web(stream: TcpStream) {
	let io = TokioIo::new(stream);
	if let Err(e) = auto::Builder::new(TokioExecutor::new())
		.serve_connection(io, service_fn(ser))
		.await
	{
		error!("{e}");
	}
}

pub async fn process_live(stream: TcpStream) {
	let io = TokioIo::new(stream);
	if let Err(e) = auto::Builder::new(TokioExecutor::new())
		.serve_connection(io, service_fn(reload))
		.await
	{
		error!("{e}");
	}
}

// use async_stream;
// use reqwest::Body;

async fn reload(r: Request<hyper::body::Incoming>) -> Result<Response<reqwest::Body>, Infallible> {
	let messages = vec![
		"Hello",
		"<meta http-equiv=\"refresh\" content=\"1\">",
		"<script>console.log('oi')</script>",
		"World",
		"From",
		"Rust",
	];
	// testar com testo pelo send refresh, pois ele ta empilhando os dados, pode também mandar
	// um html de reload
	//  unfold: https://docs.rs/futures-util-preview/latest/futures_util/stream/fn.unfold.html
	// body https://docs.rs/reqwest/latest/reqwest/struct.Body.html
	// notify  https://docs.rs/notify/latest/notify/

	let stre = futures_util::stream::unfold(messages.into_iter(), |mut iter| async {
		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
		if let Some(msg) = iter.next() {
			let msg = format!("{}\n\n", msg);
			// organizar melhor isso aqui
			return Some((Ok::<Bytes, String>(Bytes::from(msg)), iter));
		}
		return None;
	});

	let t = reqwest::Body::wrap_stream(stre);
	return Ok(Response::builder()
		.header("Content-Type", "text/event-stream")
		.header("Cache-Control", "no-cache")
		.header("Connection", "keep-alive")
		.body(t)
		.unwrap());
}

fn send_refresh() -> Response<reqwest::Body> {
	todo!();
}

/*
Full https://docs.rs/http-body-util/latest/http_body_util/struct.Full.html
Basicamente o full é oque vai representar o body para funcionar com o hyper
Response https://docs.rs/http/1.0.0/http/response/struct.Response.html
request https://docs.rs/http/1.0.0/http/request/struct.Request.html
Bytes https://docs.rs/hyper/latest/hyper/body/struct.Bytes.html
*/
async fn ser(r: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
	let uri = r.uri();
	debug!("Uri '{:?}'", uri);
	let p = Path::new(uri.path().trim_start_matches('/'));
	debug!("Path {:?}", p);
	if p.try_exists().is_ok() {
		// if let Some(n) = p.to_str() {
		if p.is_file() {
			info!("Requisitando {:?}", p);
			let v = read_fileb(p).await;
			return Ok(Response::new(Full::new(v)));
		} else if p.is_dir() {
			// deve ter is_dir() pois ele pode acabar entrando aqui se o
			// try_exist falhar por alguma falta de permissão
			// criar o html do diretorio
			if let Ok(page_dir) = read_dirb(p).await {
				info!("Pagina '{:?}' requisitada", p);
				return Ok(Response::new(Full::new(page_dir)));
			}
		}
		// }
	}
	warn!("Path não encontrado");
	// retornar msg de erro
	let error_page: maud::Markup = html! {
	    h1 { "Resource " (format!("{:?}", p)) " Not Found" }
	};
	Ok(Response::new(Full::new(Bytes::from(error_page.into_string()))))
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
		"Nao deu para ler o diretorio",
	))
}
