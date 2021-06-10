use std::{io, thread};

use log::info;

use serde_json::json;
use actix_files as fs;
use actix_web::{get, web, App, HttpServer, HttpResponse};

use crate::core::WeakFeederCore;
use super::{WeakDaemonCore, DaemonCore};

use handlebars::Handlebars;


pub struct Web {
	weak_core: Option<WeakFeederCore>,
	weak_daemon: Option<WeakDaemonCore>
}

impl Web {
	pub fn new() -> Self {
		Self {
			weak_core: None,
			weak_daemon: None
		}
	}

	pub fn listen(&mut self, weak_core: WeakFeederCore, weak_daemon: WeakDaemonCore) -> io::Result<()> {
		self.weak_daemon = Some(weak_daemon);
		self.weak_core = Some(weak_core);

		info!("Running HTTP + WS Server");


		let daemon_ref = web::Data::new(self.weak_daemon.as_ref().unwrap().clone());
		let core_ref = web::Data::new(self.weak_core.as_ref().unwrap().clone());

		thread::spawn(move || {
			let http_server = HttpServer::new(move || {
				App::new()
				.register_data(daemon_ref.clone())
				.register_data(core_ref.clone())
				.service(index)
			})
			.bind("127.0.0.1:8080").expect("HttpServer.bind");

			http_server.run().expect("HttpServer.run");
		});

		Ok(())
	}

	pub fn get_daemon(&self) -> DaemonCore {
		self.weak_daemon.as_ref().unwrap().upgrade().unwrap()
	}
}


#[get("/")]
fn index(hb: web::Data<Handlebars>) -> HttpResponse {
	HttpResponse::Ok().text("yes".into())
}