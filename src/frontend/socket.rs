use std::ops::Deref;
use std::time::{Duration, Instant};

use serde_json::{Value, to_string, json};

use actix::prelude::*;
use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::rpc::Object2CoreNotification;
use super::rpc::{Core2FrontNotification};
use crate::core::WeakFeederCore;
use crate::error::Error;
use crate::types::MessageId;


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub fn socket_index(weak_core: web::Data<WeakFeederCore>, r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
	ws::start(WebSocket::new(weak_core.deref().clone()), &r, stream)
}


pub struct WebSocket {
	weak_core: WeakFeederCore,
	hb: Instant,
}

impl Actor for WebSocket {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		self.on_start(ctx);
		self.hb(ctx);
	}
}


impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocket {
	fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
		use ws::Message::*;

		match msg {
			Nop => (),
			Close(_) => ctx.stop(),

			Ping(msg) => {
				self.hb = Instant::now();
				ctx.pong(&msg);
			}
			Pong(_) => {
				self.hb = Instant::now();
			}

			Text(text) => {
				let mut wrapper = WebsocketWrapper::new(ctx);
				if let Err(e) = handle_text(&mut wrapper, &mut self.weak_core, text) {
					eprintln!("handle_text: {}", e);
					wrapper._respond(None, Err(e));
				}
			}

			Binary(bin) => {
				let mut wrapper = WebsocketWrapper::new(ctx);
				if let Err(e) = handle_binary(&mut wrapper, &mut self.weak_core, bin.as_ref()) {
					eprintln!("handle_binary: {}", e);
				}
			}
		}
	}
}


impl WebSocket {
	fn new(weak_core: WeakFeederCore) -> Self {
		WebSocket { weak_core, hb: Instant::now() }
	}

	fn on_start(&self, _ctx: &mut <Self as Actor>::Context) {
		// let init = Core2FrontNotification::Init{};
		// ctx.text(serde_json::to_string(&init).unwrap());
	}

	fn hb(&self, ctx: &mut <Self as Actor>::Context) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				println!("Websocket Client heartbeat failed, disconnecting!");

				ctx.stop();

				return;
			}

			ctx.ping("");
		});
	}
}

fn handle_text(
	ctx: &mut WebsocketWrapper, weak_core: &mut WeakFeederCore, text: String
) -> Result<(), Error> {
	let derived = serde_json::from_str(&text)?;

	if let Object2CoreNotification::Frontend { message_id, command } = derived {
		weak_core.handle_frontend(ctx, message_id, command)?;
	}

	Ok(())
}


fn handle_binary(
	_ctx: &mut WebsocketWrapper, _weak_core: &mut WeakFeederCore, binary: &[u8]
) -> Result<(), Error> {
	println!("Binary: {:?}", binary);

	Ok(())
}


pub type WebSocketContext = ws::WebsocketContext<WebSocket>;

pub struct WebsocketWrapper<'a> {
	ctx: &'a mut WebSocketContext
}

impl<'a> WebsocketWrapper<'a> {
	pub fn new(ctx: &'a mut WebSocketContext) -> Self {
		Self { ctx }
	}

	pub fn respond_with(&mut self, message_id: Option<MessageId>, response: Core2FrontNotification) {
		match message_id {
			Some(mid) => self.respond_request(mid, response),
			None => self.respond_notification(response)
		}
	}


	pub fn respond_request(&mut self, message_id: MessageId, response: Core2FrontNotification) {
		self.respond_request_value(message_id, serde_json::to_value(response).map_err(|e| e.into()))
	}

	pub fn respond_notification(&mut self, response: Core2FrontNotification) {
		self.respond_notification_value(serde_json::to_value(response).map_err(|e| e.into()))
	}


	pub fn respond_request_value(&mut self, message_id: MessageId, response: Result<Value, Error>) {
		self._respond(Some(message_id), response);
	}

	pub fn respond_notification_value(&mut self, response: Result<Value, Error>) {
		self._respond(None, response);
	}


	pub fn _respond(&mut self, message_id: Option<MessageId>, response: Result<Value, Error>) {
		self.ctx.text(
			to_string(
				&match response {
					Ok(value) => {
						json!({
							"message_id": message_id,
							"result": value
						})
					}

					Err(err) => {
						json!({
							"message_id": message_id,
							"error": format!("{}", err)
						})
					}
				}
			).unwrap()
		);
	}
}