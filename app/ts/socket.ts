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

	public send(name: string, opts?: Obj<any>, response?: ResponseFunc<any>) {
		let message_id = this.next_msg_id();

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

		if (response != undefined) {
			this.awaiting_response.push({
				msg_id: message_id,
				resp_func: response,
				sent: Date.now(),
				timeout_seconds: 60 * 5
			});
		} else {
			// @ts-ignore
			delete wrapper.params['message_id'];
		}

		if (this.debug) console.log(JSON.stringify(method, null, 4));

		// console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper, null, 4));
	}
}


// Sending

// cat
export function send_create_category(name: string, cat_id: number, cb?: ResponseFunc<CreateCategoryResponse>) {
	let opts = {
		name: name,
		position: cat_id
	};

	app.socket.send('add_category', opts, cb);
}

export function send_remove_category(cat_feed_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		id: cat_feed_id
	};

	app.socket.send('remove_category', opts, cb);
}

export function send_edit_category(id: number, editing: ModelEditCategory,  cb?: ResponseFunc<EditListenerResponse>) {
	let opts = {
		id: id,
		editing: editing
	};

	app.socket.send('edit_category', opts, cb);
}

export function send_get_category_list(cb?: ResponseFunc<CategoryListResponse>) {
	app.socket.send('category_list', {}, cb);
}

export function send_add_feed_to_category(feed_id: number, category_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		feed_id: feed_id,
		category_id: category_id
	};

	app.socket.send('add_feed_category', opts, cb);
}

export function send_remove_feed_from_category(cat_feed_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	let opts = {
		id: cat_feed_id
	};

	app.socket.send('remove_feed_category', opts, cb);
}


// items
export function send_get_item_list(category_id: Nullable<number>, skip_count?: number, item_count?: number, cb?: ResponseFunc<ItemListResponse>) {
	let opts = {
		category_id,
		item_count,
		skip_count
	};

	app.socket.send('item_list', opts, cb);
}

// listeners
export function send_get_feed_list(cb?: ResponseFunc<FeedListResponse>) {
	app.socket.send('feed_list', {}, cb);
}

export function send_create_listener(url: string, custom_item_id: Nullable<number>, cb?: ResponseFunc<CreateListenerResponse>) {
	let opts = {
		url: url,
		custom_item_id: custom_item_id
	};

	app.socket.send('add_listener', opts, cb);
}

export function send_edit_listener(id: number, editing: ModelEditListener,  cb?: ResponseFunc<EditListenerResponse>) {
	let opts = {
		id: id,
		editing: editing
	};

	app.socket.send('edit_listener', opts, cb);
}

export function send_remove_listener(id: number, rem_stored: boolean,  cb?: ResponseFunc<RemoveListenerResponse>) {
	let opts = {
		id: id,
		rem_stored: rem_stored
	};

	app.socket.send('remove_listener', opts, cb);
}


// Editor / Custom Item

export function send_get_webpage_source(url: string, cb: ResponseFunc<GetWebpageResponse>) {
	app.socket.send('get_webpage', { url }, cb);
}

// Custom Items

export function send_get_custom_items_list(cb?: ResponseFunc<CustomItemListResponse>) {
	app.socket.send('custom_item_list', {}, cb);
}

// export function send_update_custom_item(id: number, item: ModelCustomItem, cb: ResponseFunc<any>) {
// 	app.socket.send_response('update_custom_item', {
// 		id,
// 		item
// 	}, cb);
// }

export function send_new_custom_item(item: ModelCustomItem, cb: ResponseFunc<CreateCustomItemResponse>) {
	app.socket.send('new_custom_item', {
		item
	}, cb);
}


// Filters / Feed Filters
export function send_get_filter_list(cb?: ResponseFunc<FilterListResponse>) {
	app.socket.send('filter_list', {}, cb);
}

export function send_new_filter(title: string, filter: rust.Optional<rust.EnumObject>, cb: ResponseFunc<any>) {
	app.socket.send('new_filter', {
		title,
		filter
	}, cb);
}

export function send_update_filter(id: number, title: string, filter: rust.Optional<rust.EnumObject>, cb: ResponseFunc<any>) {
	app.socket.send('update_filter', {
		id,
		title,
		filter
	}, cb);
}

export function send_remove_filter(id: number, cb: ResponseFunc<any>) {
	app.socket.send('remove_filter', {
		id
	}, cb);
}


export function send_new_feed_filter(feed_id: number, filter_id: number, cb: ResponseFunc<any>) {
	app.socket.send('new_feed_filter', {
		feed_id,
		filter_id
	}, cb);
}

export function send_remove_feed_filter(feed_id: number, filter_id: number, cb: ResponseFunc<any>) {
	app.socket.send('remove_filter', {
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

	app.socket.send('feed_updates', opts, cb);
}



/// WATCHING

export function send_get_watching_history_list(watch_id: Nullable<number>, skip_count?: number, item_count?: number, cb?: ResponseFunc<ItemListResponse>) {
	let opts = {
		watch_id,
		item_count,
		skip_count
	};

	app.socket.send('watching_item_list', opts, cb);
}

export function send_get_watcher_list(cb: ResponseFunc<WatcherListResponse>) {
	app.socket.send('watcher_list', {}, cb);
}

export function send_create_watcher(url: string, custom_item_id: Nullable<number>, cb?: ResponseFunc<CreateListenerResponse>) {
	let opts = {
		url: url,
		custom_item_id: custom_item_id
	};

	app.socket.send('add_watcher', opts, cb);
}

export function send_edit_watcher(id: number, editing: ModelEditListener,  cb?: ResponseFunc<EditListenerResponse>) {
	let opts = {
		id: id,
		editing: editing
	};

	app.socket.send('edit_watcher', opts, cb);
}

export function send_remove_watcher(id: number, rem_stored: boolean,  cb?: ResponseFunc<RemoveListenerResponse>) {
	let opts = {
		id: id,
		rem_stored: rem_stored
	};

	app.socket.send('remove_watcher', opts, cb);
}

export function send_test_watcher(url: string, parser: Nullable<any>, cb: ResponseFunc<CreateListenerResponse>) {
	let opts = {
		url,
		parser
	};

	app.socket.send('test_watcher', opts, cb);
}

export function send_new_watch_parser(item: ModelWatchParser, cb: ResponseFunc<CreateCustomItemResponse>) {
	app.socket.send('new_watch_parser', {
		item
	}, cb);
}

export function send_get_watch_parser_list(cb: ResponseFunc<WatchParserListResponse>) {
	app.socket.send('watch_parser_list', {}, cb);
}