import { EventEmitter, Injectable, Output } from '@angular/core';
import { FeedItem } from './item/feed-item';
import { FeedListener } from './item/feed-listener';
import { WebsocketService } from './websocket.service';

@Injectable({
	providedIn: 'root'
})

export class BackgroundService {
	public feed_list: FeedListener[] = [];

	public filter_list: FilterGroupListener[] = [];

	public watching_listeners = <[ModelWatcher, ModelWatchHistory][]>[];

	public custom_items: ModelCustomItem[] = [];
	public watch_parser: ModelWatchParser[] = [];

	@Output() new_feed_items = new EventEmitter<ItemListResponse>();

	constructor(private websocket: WebsocketService) {}

	// Initial call when loading website.
	private async init_feeds() {
		let feed_list_resp = await this.websocket.send_get_feed_list();

		this.feed_list = feed_list_resp.items.map(v => new FeedListener(v));

		this.filter_list = (await this.websocket.send_get_filter_list()).items;
		this.custom_items = (await this.websocket.send_get_custom_items_list()).items;
		this.watch_parser = (await this.websocket.send_get_watch_parser_list()).items;

		let watcher_list_resp = await this.websocket.send_get_watcher_list();
		this.watching_listeners = watcher_list_resp.items;
	}

	public init(): void {
		Notification.requestPermission();

		this.init_feeds().catch(console.error);

		console.log(this);

		this.websocket.socket_update_listener.subscribe(update => {
			if (update.feed_items_count != 0) {
				console.log('New feed items');

				this.websocket.send_get_item_list(null, null, 0, update.feed_items_count)
				.then(resp => {
					console.log('[BG]: New Items:', resp);
					this.new_feed_items.emit(resp);
				})
				.catch(e => console.error('[BG]: Grabbing Feed Items List', e));
			}

			if (update.watch_items_count != 0) {
				new Notification(`Received ${update.watch_items_count} new watch update(s).`);

				this.websocket.send_get_watcher_list()
				.then(resp => {
					console.log('[BG]: Watching:', resp);
					this.watching_listeners = resp.items;
				})
				.catch(e => console.error('[BG]: Grabbing Watcher List', e));
			}
		});
	}

	get_feed_by_id(id: number): Optional<ModelListener> {
		return this.feed_list.find(f => f.id == id);
	}

	get_feed_by_url(url: string): Optional<ModelListener> {
		return this.feed_list.find(i => i.url == url);
	}

	get_enabled_filters_by_feed_id(id: number): FilterGroupListener[] {
		return this.filter_list.filter(v => v.feeds.includes(id));
	}

	get_disabled_filters_by_feed_id(id: number): FilterGroupListener[] {
		return this.filter_list.filter(v => !v.feeds.includes(id));
	}

	has_notification_perms(): boolean {
		return Notification.permission == 'granted';
	}
}