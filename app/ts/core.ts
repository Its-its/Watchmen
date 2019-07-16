type Obj<I> = { [name: string]: I };

type Nullable<I> = I | null;

interface Response {
	[name: string]: any;

	message_id?: number;
	error?: any;
	result?: any;
}

type ResponseFunc = (error: any, value: any) => any;

interface AwaitingReponse {
	sent: number,
	timeout_seconds: number,

	msg_id: number,
	resp_func?: ResponseFunc
}


class SocketManager {
	socket = new WebSocket("ws://" + window.location.host + "/ws/");

	last_message_id = 0;

	awaiting_response: AwaitingReponse[] = [];

	constructor() {
		this.socket.onmessage = event => this.onMessage(JSON.parse(event.data));
	}

	public onMessage(response: Response) {
		switch(response.method) {
			case 'init': {
				send_update('test');
				break;
			}

			default: {
				if (response.message_id != null) {
					this.update_response(response);
				} else {
					console.log(response);
				}
			}
		}
	}

	public next_msg_id(): number {
		return this.last_message_id++;
	}

	public update_response(value: Response) {
		for (var i = 0; i < this.awaiting_response.length; i++) {
			var resp = this.awaiting_response[i];

			if (resp.msg_id == value.message_id) {
				if (resp.resp_func) resp.resp_func(value.error, value.result);
				this.awaiting_response.splice(i, 1);
				break;
			}
		}
	}

	//

	public send_response(name: string, opts: Obj<any>, resp?: ResponseFunc) {
		this.invoke_response(
			name,
			opts,
			resp
		);
	}

	public send_notification(name: string, opts: Obj<any>) {
		this.invoke_notification(
			name,
			opts
		);
	}


	public invoke_response(name: string, opts: Obj<any>, response?: ResponseFunc) {
		var message_id = this.next_msg_id();

		this.awaiting_response.push({
			msg_id: message_id,
			resp_func: response,
			sent: Date.now(),
			timeout_seconds: 60 * 5
		});

		var method = {
			"method": name,
			"params": {
				// "command": opts || {}
			}
		};

		var wrapper = {
			"method": "frontend",
			"params": {
				"message_id": message_id,
				"command": method
			}
		};

		console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper));
	}


	public invoke_notification(name: string, opts?: Obj<any>) {
		var method = {
			"method": name,
			"params": {
				// "command": opts || {}
			}
		};

		var wrapper = {
			"method": "frontend",
			"params": {
				"command": method
			}
		};

		console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper));
	}
}

const socket = new SocketManager();





// Sending

function send_update(name: string, opts?: Obj<any>, resp?: ResponseFunc) {
	socket.send_response('update', {
		method: name,
		params: opts || {}
	}, resp);
}
