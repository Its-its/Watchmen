use std::{io, thread};

use log::info;

use actix_rt::System;
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

		let frontend_ref = self.weak_frontend.as_ref().unwrap().clone();
		let core_ref = self.weak_core.as_ref().unwrap().clone();

		thread::spawn(move || {
			let mut sys = System::new("HTTP Server");

			let server = HttpServer::new(move || {
				App::new()
				.data({
					let mut handlebars = Handlebars::new();
					handlebars.register_templates_directory(".hbs", "./app/views").expect("register_templates_dirs");
					handlebars
				})
				.data(frontend_ref.clone())
				.data(core_ref.clone())
				.service(index)
				.service(scraper_editor)
				.service(fs::Files::new("/script", "./app/compiled/js"))
				.service(fs::Files::new("/style", "./app/compiled/css"))
				.service(web::resource("/ws/").route(web::get().to(socket_index)))
			})
			.bind("0.0.0.0:8080")
			.unwrap()
			.run();

			let _ = sys.block_on(server);
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


// Scaper

#[get("/scraper/editor")]
fn scraper_editor(hb: web::Data<Handlebars>) -> HttpResponse {
	let data = json!({});

	let body = hb.render("scraper_editor", &data).unwrap();

	HttpResponse::Ok().body(body)
}