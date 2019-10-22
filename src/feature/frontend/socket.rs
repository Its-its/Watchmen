use std::ops::Deref;
use std::time::{Duration, Instant};

use log::{info, error};

use serde_json::{Value, to_string, json};

use actix::prelude::*;
use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::rpc::Object2CoreNotification;
use crate::feature::ResponseWrapper;
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
					error!("handle_text: {}", e);
					wrapper.respond(None, Err(e));
				}
			}

			Binary(bin) => {
				let mut wrapper = WebsocketWrapper::new(ctx);
				if let Err(e) = handle_binary(&mut wrapper, &mut self.weak_core, bin.as_ref()) {
					error!("handle_binary: {}", e);
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
				info!("Websocket Client heartbeat failed, disconnecting!");

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
		weak_core.handle_response(ctx, message_id, command)?;
	}

	Ok(())
}


fn handle_binary(
	_ctx: &mut WebsocketWrapper, _weak_core: &mut WeakFeederCore, binary: &[u8]
) -> Result<(), Error> {
	info!("Binary: {:?}", binary);

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
}

impl<'a> ResponseWrapper for WebsocketWrapper<'a> {
	fn respond(&mut self, message_id: Option<MessageId>, response: Result<Value, Error>) {
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