use std::{io, thread};

use log::info;

use serde_json::json;
use actix_files as fs;
use actix_web::{get, web, App, HttpServer, HttpResponse};

use crate::core::WeakFeederCore;
use super::{WeakFrontendCore, FrontendCore};
use super::socket::socket_index;


use handlebars::Handlebars;



pub struct Web {
	weak_core: Option<WeakFeederCore>,
	weak_frontend: Option<WeakFrontendCore>
}

impl Web {
	pub fn new() -> Self {
		Self {
			weak_core: None,
			weak_frontend: None
		}
	}

	pub fn listen(&mut self, weak_core: WeakFeederCore, weak_frontend: WeakFrontendCore) -> io::Result<()> {
		self.weak_frontend = Some(weak_frontend);
		self.weak_core = Some(weak_core);

		info!("Running HTTP + WS Server");

		let mut handlebars = Handlebars::new();
		handlebars.register_templates_directory(".hbs", "./app/views").expect("register_templates_dirs");

		let handlebars_ref = web::Data::new(handlebars);
		let frontend_ref = web::Data::new(self.weak_frontend.as_ref().unwrap().clone());
		let core_ref = web::Data::new(self.weak_core.as_ref().unwrap().clone());

		thread::spawn(move || {
			let http_server = HttpServer::new(move || {
				App::new()
				.register_data(handlebars_ref.clone())
				.register_data(frontend_ref.clone())
				.register_data(core_ref.clone())
				.service(index)
				.service(fs::Files::new("/script", "./app/compiled/js"))
				.service(fs::Files::new("/style", "./app/compiled/css"))
				.service(web::resource("/ws/").route(web::get().to(socket_index)))
			})
			.bind("127.0.0.1:8080").expect("HttpServer.bind");

			http_server.run().expect("HttpServer.run");
		});

		Ok(())
	}

	pub fn get_frontend(&self) -> FrontendCore {
		self.weak_frontend.as_ref().unwrap().upgrade().unwrap()
	}
}


#[get("/")]
fn index(hb: web::Data<Handlebars>) -> HttpResponse {
	let data = json!({});

	let body = hb.render("index", &data).unwrap();

	HttpResponse::Ok().body(body)
}