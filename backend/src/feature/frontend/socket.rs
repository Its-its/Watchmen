use std::time::{Duration, Instant};

use log::{info, error};

use serde_json::{Value, to_string, json};

use actix::prelude::*;
use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, WebsocketContext};

use crate::rpc::Object2CoreNotification;
use crate::feature::ResponseWrapper;
use crate::core::WeakFeederCore;
use crate::error::Error;
use crate::types::MessageId;


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);


pub async fn socket_index(weak_core: web::Data<WeakFeederCore>, r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
	ws::start(WebSocket::new(weak_core.as_ref().clone()), &r, stream)
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct Line(String);

impl Handler<Line> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: Line, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
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


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		use ws::Message;

		match msg {
			Ok(Message::Ping(msg)) => {
				self.hb = Instant::now();
				ctx.pong(&msg);
			}

			Ok(Message::Pong(_) )=> {
				self.hb = Instant::now();
			}

			Ok(Message::Text(text)) => {
				if let Err(e) = self.handle_text(ctx, &text) {
					match e {
						Error::Json(e) => {
							error!("handle_text JSON: {}", e);

							let line = text.split('\n').nth(e.line() - 1);

							if let Some(line) = line {
								println!("Line: '{}'", line);
							}
						}

						e  => {
							error!("handle_text: {}", e);
						}
					}
				}
			}

			Ok(Message::Binary(bin)) => {
				let recipient = ctx.address().recipient();
				let mut wrapper = WebsocketWrapper::new(recipient);

				if let Err(e) = handle_binary(&mut wrapper, &self.weak_core, bin.as_ref()) {
					error!("handle_binary: {}", e);
				}
			}

			_ => ctx.stop()
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

			ctx.ping(&[]);
		});
	}

	fn handle_text(
		&self,
		ctx: &mut WebsocketContext<WebSocket>,
		text: &str
	) -> Result<(), Error> {
		let derived = serde_json::from_str(text)?;

		if let Object2CoreNotification::Frontend { message_id, command } = derived {
			let recipient = ctx.address().recipient();
			let mut wrap = WebsocketWrapper::new(recipient);

			let weak_core = self.weak_core.clone();

			let future = async move {
				if let Err(e) = weak_core.handle_response(&mut wrap, message_id, command).await {
					wrap.respond(message_id, Err(e));
				}
			};

			future.into_actor(self).spawn(ctx);
		}

		Ok(())
	}
}


fn handle_binary(
	_ctx: &mut WebsocketWrapper,
	_weak_core: &WeakFeederCore,
	binary: &[u8]
) -> Result<(), Error> {
	info!("Binary: {:?}", binary);

	Ok(())
}


pub type WebSocketContext = ws::WebsocketContext<WebSocket>;

pub struct WebsocketWrapper {
	pub recipient: Recipient<Line>
}

impl WebsocketWrapper {
	pub fn new(recipient: Recipient<Line>) -> Self {
		Self { recipient }
	}
}

impl ResponseWrapper for WebsocketWrapper {
	fn respond(&mut self, message_id: Option<MessageId>, response: Result<Value, Error>) {
		self.recipient.do_send(
			Line(to_string(
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
			).unwrap())
		).unwrap();
	}
}