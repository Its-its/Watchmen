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

		setInterval(() => {
			for (let i = this.awaiting_response.length - 1; i >= 0; i--) {
				const resp = this.awaiting_response[i];

				if (Date.now() > resp.sent + (resp.timeout_seconds * 1000)) {
					this.awaiting_response.splice(i, 1);
				}
			}
		}, 5 * 1000);
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

	public send(name: string, opts?: Obj<any>): Promise<any> {
		return new Promise((resolve, reject) => {
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

			this.awaiting_response.push({
				msg_id: message_id,
				resp_func: function(err, value) {
					if (err != null) return reject(err);
					else resolve(value);
				},
				sent: Date.now(),
				timeout_seconds: 60 * 5
			});

			if (this.debug) console.log(JSON.stringify(method, null, 4));

			this.socket.send(JSON.stringify(wrapper, null, 4));
		});
	}
}


// Sending

// cat
export function send_create_category(name: string, cat_id: number): Promise<CreateCategoryResponse> {
	let opts = {
		name: name,
		position: cat_id
	};

	return app.socket.send('add_category', opts);
}

export function send_remove_category(cat_feed_id: number): Promise<AddCategoryFeedResponse> {
	let opts = {
		id: cat_feed_id
	};

	return app.socket.send('remove_category', opts);
}

export function send_edit_category(id: number, editing: ModelEditCategory): Promise<EditListenerResponse> {
	let opts = {
		id: id,
		editing: editing
	};

	return app.socket.send('edit_category', opts);
}

export function send_get_category_list(): Promise<CategoryListResponse> {
	return app.socket.send('category_list', {});
}

export function send_add_feed_to_category(feed_id: number, category_id: number): Promise<AddCategoryFeedResponse> {
	let opts = {
		feed_id: feed_id,
		category_id: category_id
	};

	return app.socket.send('add_feed_category', opts);
}

export function send_remove_feed_from_category(cat_feed_id: number): Promise<AddCategoryFeedResponse> {
	let opts = {
		id: cat_feed_id
	};

	return app.socket.send('remove_feed_category', opts);
}


// items
export function send_get_item_list(category_id: Nullable<number>, skip_count?: number, item_count?: number): Promise<ItemListResponse> {
	let opts = {
		category_id,
		item_count,
		skip_count
	};

	return app.socket.send('item_list', opts);
}

// listeners
export function send_get_feed_list(): Promise<FeedListResponse> {
	return app.socket.send('feed_list', {});
}

export function send_create_listener(url: string, custom_item_id: Nullable<number>): Promise<CreateListenerResponse> {
	let opts = {
		url: url,
		custom_item_id: custom_item_id
	};

	return app.socket.send('add_listener', opts);
}

export function send_edit_listener(id: number, editing: ModelEditListener): Promise<EditListenerResponse> {
	let opts = {
		id: id,
		editing: editing
	};

	return app.socket.send('edit_listener', opts);
}

export function send_remove_listener(id: number, rem_stored: boolean): Promise<RemoveListenerResponse> {
	let opts = {
		id: id,
		rem_stored: rem_stored
	};

	return app.socket.send('remove_listener', opts);
}


// Editor / Custom Item

export function send_get_webpage_source(url: string): Promise<GetWebpageResponse> {
	return app.socket.send('get_webpage', { url });
}

// Custom Items

export function send_get_custom_items_list(): Promise<CustomItemListResponse> {
	return app.socket.send('custom_item_list', {});
}

// export function send_update_custom_item(id: number, item: ModelCustomItem): Promise<any> {
// 	return app.socket.send_response('update_custom_item', {
// 		id,
// 		item
// 	});
// }

export function send_new_custom_item(item: ModelCustomItem): Promise<CreateCustomItemResponse> {
	return app.socket.send('new_custom_item', {
		item
	});
}


// Filters / Feed Filters
export function send_get_filter_list(): Promise<FilterListResponse> {
	return app.socket.send('filter_list', {});
}

export function send_new_filter(title: string, filter: rust.Optional<rust.EnumObject>): Promise<any> {
	return app.socket.send('new_filter', {
		title,
		filter
	});
}

export function send_update_filter(id: number, title: string, filter: rust.Optional<rust.EnumObject>): Promise<any> {
	return app.socket.send('update_filter', {
		id,
		title,
		filter
	});
}

export function send_remove_filter(id: number): Promise<any> {
	return app.socket.send('remove_filter', {
		id
	});
}


export function send_new_feed_filter(feed_id: number, filter_id: number): Promise<any> {
	return app.socket.send('new_feed_filter', {
		feed_id,
		filter_id
	});
}

export function send_remove_feed_filter(feed_id: number, filter_id: number): Promise<any> {
	return app.socket.send('remove_filter', {
		feed_id,
		filter_id
	});
}

// other
export function send_get_updates_since(since_timestamp: number): Promise<UpdatesResponse> {
	// Gets the amount of feeds that are newer than feed_timestamp.
	let opts = {
		since: since_timestamp
	};

	return app.socket.send('feed_updates', opts);
}



/// WATCHING

export function send_get_watch_history_list(watch_id: Nullable<number>, skip_count: Optional<number>, item_count: Optional<number>): Promise<WatchHistoryListResponse> {
	let opts = {
		watch_id,
		item_count,
		skip_count
	};

	return app.socket.send('watch_history_list', opts);
}

export function send_get_watcher_list(): Promise<WatcherListResponse> {
	return app.socket.send('watcher_list', {});
}

export function send_create_watcher(url: string, custom_item_id: Nullable<number>): Promise<CreateListenerResponse> {
	let opts = {
		url: url,
		custom_item_id: custom_item_id
	};

	return app.socket.send('add_watcher', opts);
}

export function send_edit_watcher(id: number, editing: ModelEditWatcher): Promise<EditListenerResponse> {
	let opts = {
		id: id,
		editing: editing
	};

	return app.socket.send('edit_watcher', opts);
}

export function send_remove_watcher(id: number, rem_stored: boolean): Promise<RemoveListenerResponse> {
	let opts = {
		id: id,
		rem_stored: rem_stored
	};

	return app.socket.send('remove_watcher', opts);
}

export function send_test_watcher(url: string, parser: Nullable<any>): Promise<CreateListenerResponse> {
	let opts = {
		url,
		parser
	};

	return app.socket.send('test_watcher', opts);
}

export function send_new_watch_parser(item: ModelWatchParser): Promise<CreateCustomItemResponse> {
	return app.socket.send('new_watch_parser', {
		item
	});
}

export function send_get_watch_parser_list(): Promise<WatchParserListResponse> {
	return app.socket.send('watch_parser_list', {});
}