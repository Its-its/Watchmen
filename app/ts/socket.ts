import app from './core';
import { RustEnum } from './util/rust';

export default class SocketManager {
	socket = new WebSocket("ws://" + window.location.host + "/ws/");

	debug = true;

	last_message_id = 0;

	awaiting_response: AwaitingReponse[] = [];

	constructor() {
		this.socket.addEventListener('message', event => this.onMessage(JSON.parse(event.data)));
		this.socket.addEventListener('open', event => this.on_open(event));
	}

	public is_open(): boolean {
		return this.socket.readyState == this.socket.OPEN;
	}


	public on_open(_: Event): any {
		app.on_connection_open();
	}

	public onMessage(resp: SocketResponse) {
		// Only return if it's not going to be sending to fn.
		if (resp.error != null && resp.message_id == null) {
			return console.error(resp);
		}

		if (resp.message_id != null) {
			return this.update_response(resp);
		} else {
			console.log('Default:', resp);
		}
	}


	public next_msg_id(): number {
		return this.last_message_id++;
	}

	public update_response(value: SocketResponse) {
		for (let i = 0; i < this.awaiting_response.length; i++) {
			let resp = this.awaiting_response[i];

			if (resp.msg_id == value.message_id) {
				if (resp.resp_func != null) {
					let result = value.result;

					if (result != null) {
						resp.resp_func(
							value.error,
							result.params,
							result.method
						);
					} else {
						resp.resp_func(value.error);
					}
				}

				this.awaiting_response.splice(i, 1);
				break;
			}
		}
	}

	//

	public send_response(name: string, opts: Obj<any>, response: ResponseFunc<any>) {
		let message_id = this.next_msg_id();

		this.awaiting_response.push({
			msg_id: message_id,
			resp_func: response,
			sent: Date.now(),
			timeout_seconds: 60 * 5
		});

		let method = {
			"method": name,
			"params": opts
		};

		let wrapper = {
			"method": "frontend",
			"params": {
				"message_id": message_id,
				"command": method
			}
		};

		if (this.debug) console.log(JSON.stringify(method, null, 4));

		// console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper, null, 4));
	}

	public send_notification(name: string, opts?: Obj<any>) {
		let method = {
			"method": name,
			"params": opts
		};

		let wrapper = {
			"method": "frontend",
			"params": {
				"command": method
			}
		};

		// console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper));
	}
}


// Sending

// cat
export function send_create_category(name: string, cat_id: number, cb?: ResponseFunc<CreateCategoryResponse>) {
	let opts = {
		name: name,
		position: cat_id
	};

	if (cb == null) {
		app.socket.send_notification('add_category', opts);
	} else {
		app.socket.send_response('add_category', opts, cb);
	}
}

export function send_remove_category(cat_feed_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		id: cat_feed_id
	};

	if (cb == null) {
		app.socket.send_notification('remove_category', opts);
	} else {
		app.socket.send_response('remove_category', opts, cb);
	}
}

export function send_edit_category(id: number, editing: ModelEditCategory,  cb?: ResponseFunc<EditListenerResponse>) {
	let opts = {
		id: id,
		editing: editing
	};

	if (cb == null) {
		app.socket.send_notification('edit_category', opts);
	} else {
		app.socket.send_response('edit_category', opts, cb);
	}
}

export function send_get_category_list(cb?: ResponseFunc<CategoryListResponse>) {
	if (cb == null) {
		app.socket.send_notification('category_list');
	} else {
		app.socket.send_response('category_list', {}, cb);
	}
}

export function send_add_feed_to_category(feed_id: number, category_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		feed_id: feed_id,
		category_id: category_id
	};

	if (cb == null) {
		app.socket.send_notification('add_feed_category', opts);
	} else {
		app.socket.send_response('add_feed_category', opts, cb);
	}
}

export function send_remove_feed_from_category(cat_feed_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		id: cat_feed_id
	};

	if (cb == null) {
		app.socket.send_notification('remove_feed_category', opts);
	} else {
		app.socket.send_response('remove_feed_category', opts, cb);
	}
}


// items
export function send_get_item_list(category_id: Nullable<number>, skip?: number, items?: number, cb?: ResponseFunc<ItemListResponse>) {
	let opts = {
		category_id: category_id,
		items: items,
		skip: skip
	};

	if (cb == null) {
		app.socket.send_notification('item_list', opts);
	} else {
		app.socket.send_response('item_list', opts, cb);
	}
}

// listeners
export function send_get_feed_list(cb?: ResponseFunc<FeedListResponse>) {
	if (cb == null) {
		app.socket.send_notification('feed_list');
	} else {
		app.socket.send_response('feed_list', {}, cb);
	}
}

export function send_create_listener(url: string, custom_item_id: Nullable<number>, cb?: ResponseFunc<CreateListenerResponse>) {
	let opts = {
		url: url,
		custom_item_id: custom_item_id
	};

	if (cb == null) {
		app.socket.send_notification('add_listener', opts);
	} else {
		app.socket.send_response('add_listener', opts, cb);
	}
}

export function send_edit_listener(id: number, editing: ModelEditListener,  cb?: ResponseFunc<EditListenerResponse>) {
	let opts = {
		id: id,
		editing: editing
	};

	if (cb == null) {
		app.socket.send_notification('edit_listener', opts);
	} else {
		app.socket.send_response('edit_listener', opts, cb);
	}
}

export function send_remove_listener(id: number, rem_stored: boolean,  cb?: ResponseFunc<RemoveListenerResponse>) {
	let opts = {
		id: id,
		rem_stored: rem_stored
	};

	if (cb == null) {
		app.socket.send_notification('remove_listener', opts);
	} else {
		app.socket.send_response('remove_listener', opts, cb);
	}
}


// Editor / Custom Item

export function send_get_webpage_source(url: string, cb: ResponseFunc<GetWebpageResponse>) {
	app.socket.send_response('get_webpage', { url }, cb);
}

// Custom Items

export function send_get_custom_items_list(cb?: ResponseFunc<CustomItemListResponse>) {
	if (cb == null) {
		app.socket.send_notification('custom_item_list');
	} else {
		app.socket.send_response('custom_item_list', {}, cb);
	}
}

// export function send_update_custom_item(id: number, item: ModelCustomItem, cb: ResponseFunc<any>) {
// 	app.socket.send_response('update_custom_item', {
// 		id,
// 		item
// 	}, cb);
// }

export function send_new_custom_item(item: ModelCustomItem, cb: ResponseFunc<CreateCustomItemResponse>) {
	app.socket.send_response('new_custom_item', {
		item
	}, cb);
}


// Filters / Feed Filters
export function send_get_filter_list(cb?: ResponseFunc<FilterListResponse>) {
	if (cb == null) {
		app.socket.send_notification('filter_list');
	} else {
		app.socket.send_response('filter_list', {}, cb);
	}
}

export function send_new_filter(title: string, filter: rust.Optional<rust.EnumObject>, cb: ResponseFunc<any>) {
	app.socket.send_response('new_filter', {
		title,
		filter
	}, cb);
}

export function send_update_filter(id: number, title: string, filter: rust.Optional<rust.EnumObject>, cb: ResponseFunc<any>) {
	app.socket.send_response('update_filter', {
		id,
		title,
		filter
	}, cb);
}

export function send_remove_filter(id: number, cb: ResponseFunc<any>) {
	app.socket.send_response('remove_filter', {
		id
	}, cb);
}


export function send_new_feed_filter(feed_id: number, filter_id: number, cb: ResponseFunc<any>) {
	app.socket.send_response('new_feed_filter', {
		feed_id,
		filter_id
	}, cb);
}

export function send_remove_feed_filter(feed_id: number, filter_id: number, cb: ResponseFunc<any>) {
	app.socket.send_response('remove_filter', {
		feed_id,
		filter_id
	}, cb);
}

// other
export function send_get_updates_since(since_timestamp: number, cb?: ResponseFunc<UpdatesResponse>) {
	// Gets the amount of feeds that are newer than feed_timestamp.
	let opts = {
		since: since_timestamp
	};

	if (cb == null) {
		app.socket.send_notification('updates', opts);
	} else {
		app.socket.send_response('updates', opts, cb);
	}
}
