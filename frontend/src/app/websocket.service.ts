import { Injectable } from "@angular/core";

import { webSocket, WebSocketSubject } from "rxjs/webSocket";


@Injectable({
	providedIn: 'root'
})

export class WebsocketService {
	private subject: WebSocketSubject<any> | null = null;

	private debug = true;

	private last_message_id = 0;

	private awaiting_response: any[] = [];

	constructor() {
		this.reconnect();
	}

	private reconnect() {
		console.log('Connecting to WebSocket');

		this.subject = webSocket(`ws://${window.location.host}/ws/`);

		this.subject.asObservable()
		.subscribe({
			next: resp => {
				// Only return if it's not going to be sending to fn.
				if (resp.error != null && resp.message_id == null) {
					return console.error(resp);
				}

				if (resp.message_id != null) {
					return this.update_response(resp);
				} else {
					console.log('Default:', resp);
				}
			},

			error: e => {
				console.error('Websocket', e);

				// Continuously retries to connect even if backend is offline.
				if (e.target.readyState == WebSocket.CLOSED) {
					setTimeout(() => this.reconnect(), 5000);
				}
			},

			complete: () => {
				console.log('complete (idk what this is for)');
			}
		});
	}


	private next_msg_id(): number {
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
				resp_func: function(err: any, value: any) {
					if (err != null) return reject(err);
					else resolve(value);
				},
				sent: Date.now(),
				timeout_seconds: 60 * 5
			});

			if (this.debug) console.log(JSON.stringify(method, null, 4));

			this.subject!.next(wrapper);
		});
	}



	// Sending

	// cat
	public send_create_category(name: string, cat_id: number): Promise<CreateCategoryResponse> {
		let opts = {
			name: name,
			position: cat_id
		};

		return this.send('add_category', opts);
	}

	public send_remove_category(cat_feed_id: number): Promise<AddCategoryFeedResponse> {
		let opts = {
			id: cat_feed_id
		};

		return this.send('remove_category', opts);
	}

	public send_edit_category(id: number, editing: ModelEditCategory): Promise<EditListenerResponse> {
		let opts = {
			id: id,
			editing: editing
		};

		return this.send('edit_category', opts);
	}

	public send_get_category_list(): Promise<CategoryListResponse> {
		return this.send('category_list', {});
	}

	public send_add_feed_to_category(feed_id: number, category_id: number): Promise<AddCategoryFeedResponse> {
		let opts = {
			feed_id: feed_id,
			category_id: category_id
		};

		return this.send('add_feed_category', opts);
	}

	public send_remove_feed_from_category(cat_feed_id: number): Promise<AddCategoryFeedResponse> {
		let opts = {
			id: cat_feed_id
		};

		return this.send('remove_feed_category', opts);
	}


	// items
	public send_get_item_list(search: Nullable<string>, category_id: Nullable<number>, skip_count?: number, item_count?: number): Promise<ItemListResponse> {
		let opts = {
			search,
			category_id,
			item_count,
			skip_count
		};

		return this.send('item_list', opts);
	}

	// listeners
	public send_get_feed_list(): Promise<FeedListResponse> {
		return this.send('feed_list', {});
	}

	public send_create_listener(url: string, custom_item_id: Nullable<number>): Promise<CreateListenerResponse> {
		let opts = {
			url: url,
			custom_item_id: custom_item_id
		};

		return this.send('add_listener', opts);
	}

	public send_edit_listener(id: number, editing: ModelEditListener): Promise<EditListenerResponse> {
		let opts = {
			id: id,
			editing: editing
		};

		return this.send('edit_listener', opts);
	}

	public send_remove_listener(id: number, rem_stored: boolean): Promise<RemoveListenerResponse> {
		let opts = {
			id: id,
			rem_stored: rem_stored
		};

		return this.send('remove_listener', opts);
	}


	// Editor / Custom Item

	public send_get_webpage_source(url: string): Promise<GetWebpageResponse> {
		return this.send('get_webpage', { url });
	}

	// Custom Items

	public send_get_custom_items_list(): Promise<CustomItemListResponse> {
		return this.send('custom_item_list', {});
	}

	// public send_update_custom_item(id: number, item: ModelCustomItem): Promise<any> {
	// 	return this.send_response('update_custom_item', {
	// 		id,
	// 		item
	// 	});
	// }

	public send_new_custom_item(item: ModelCustomItem): Promise<CreateCustomItemResponse> {
		return this.send('new_custom_item', {
			item
		});
	}


	// Filters / Feed Filters
	public send_get_filter_list(): Promise<FilterListResponse> {
		return this.send('filter_list', {});
	}

	public send_new_filter(title: string, filter: rust.Optional<rust.EnumObject>): Promise<any> {
		return this.send('new_filter', {
			title,
			filter
		});
	}

	public send_update_filter(id: number, title: string, filter: rust.Optional<rust.EnumObject>): Promise<any> {
		return this.send('update_filter', {
			id,
			title,
			filter
		});
	}

	public send_remove_filter(id: number): Promise<any> {
		return this.send('remove_filter', {
			id
		});
	}


	public send_new_feed_filter(feed_id: number, filter_id: number): Promise<any> {
		return this.send('new_feed_filter', {
			feed_id,
			filter_id
		});
	}

	public send_remove_feed_filter(feed_id: number, filter_id: number): Promise<any> {
		return this.send('remove_feed_filter', {
			feed_id,
			filter_id
		});
	}

	// other
	public send_get_updates_since(since_timestamp: number): Promise<UpdatesResponse> {
		// Gets the amount of feeds that are newer than feed_timestamp.
		let opts = {
			since: since_timestamp
		};

		return this.send('feed_updates', opts);
	}



	/// WATCHING

	public send_get_watch_history_list(watch_id: Nullable<number>, skip_count: Optional<number>, item_count: Optional<number>): Promise<WatchHistoryListResponse> {
		let opts = {
			watch_id,
			item_count,
			skip_count
		};

		return this.send('watch_history_list', opts);
	}

	public send_get_watcher_list(): Promise<WatcherListResponse> {
		return this.send('watcher_list', {});
	}

	public send_create_watcher(url: string, custom_item_id: Nullable<number>): Promise<CreateListenerResponse> {
		let opts = {
			url: url,
			custom_item_id: custom_item_id
		};

		return this.send('add_watcher', opts);
	}

	public send_edit_watcher(id: number, editing: ModelEditWatcher): Promise<EditListenerResponse> {
		let opts = {
			id: id,
			editing: editing
		};

		return this.send('edit_watcher', opts);
	}

	public send_remove_watcher(id: number, rem_stored: boolean): Promise<RemoveListenerResponse> {
		let opts = {
			id: id,
			rem_stored: rem_stored
		};

		return this.send('remove_watcher', opts);
	}

	public send_test_watcher(url: string, parser: Nullable<any>): Promise<CreateListenerResponse> {
		let opts = {
			url,
			parser
		};

		return this.send('test_watcher', opts);
	}

	public send_new_watch_parser(item: ModelWatchParser): Promise<CreateWatchParserResponse> {
		return this.send('new_watch_parser', {
			item
		});
	}

	public send_update_watch_parser(id: number, item: ModelEditWatchParser): Promise<UpdateWatchParserResponse> {
		return this.send('update_watch_parser', {
			id,
			item
		});
	}

	public send_remove_watch_parser(id: number): Promise<RemoveWatchParserResponse> {
		return this.send('remove_watch_parser', {
			id
		});
	}

	public send_get_watch_parser_list(): Promise<WatchParserListResponse> {
		return this.send('watch_parser_list', {});
	}


	/// REQUEST HISTORY GROUP / ITEMS

	public send_get_request_history_group_list(skip_count: Optional<number>, item_count: Optional<number>): Promise<RequestHistoryGroupListResponse> {
		let opts = {
			item_count,
			skip_count
		};

		return this.send('request_history_list', opts);
	}

	public send_get_request_history_group_items(group_id: number): Promise<RequestHistoryItemListResponse> {
		let opts = {
			group_id
		};

		return this.send('request_history_group_items', opts);
	}
}