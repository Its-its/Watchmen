import { Injectable, Output } from '@angular/core';
import { FeedItem } from './item/feed-item';
import { FeedListener } from './item/feed-listener';
import { WebsocketService } from './websocket.service';

@Injectable({
	providedIn: 'root'
})

export class BackgroundService {
	public feed_list: FeedListener[] = [];

	public feed_items: FeedItem[] = [];

	public filter_list: FilterGroupListener[] = [];

	public watching_listeners = <[ModelWatcher, ModelWatchHistory][]>[];

	// Editor (TODO: Place in to respective files.)
	public custom_items: ModelCustomItem[] = [];
	public watch_parser: ModelWatchParser[] = [];

	public categories: ModelCategory[] = [];
	public category_feeds: ModelFeedCategory[] = [];

	viewing_category: number | null = null;
	search_params: string | null = null;

	public page_index: number = 0;
	public page_size: number = 25;

	constructor(private websocket: WebsocketService) {}

	// Initial call when loading website.
	private async init_feeds() {
		let feed_list_resp = await this.websocket.send_get_feed_list();

		this.feed_list = feed_list_resp.items.map(v => new FeedListener(v));
		console.log('Feeds:', this.feed_list);

		this.filter_list = (await this.websocket.send_get_filter_list()).items;
		this.custom_items = (await this.websocket.send_get_custom_items_list()).items;
		this.watch_parser = (await this.websocket.send_get_watch_parser_list()).items;

		let cats = await this.websocket.send_get_category_list();
		this.categories = cats.categories;
		this.category_feeds = cats.category_feeds;

		await this.reset_feeds();

		let watcher_list_resp = await this.websocket.send_get_watcher_list();
		this.watching_listeners = watcher_list_resp.items;
	}

	public init(): void {
		Notification.requestPermission();

		this.init_feeds().catch(console.error);

		console.log(this);

		// TODO: Remove. Actually utilize websockets.
		setInterval(() => {
			(async () => { // TODO: Error handling.
				this.websocket.send_get_updates_since(this.get_newest_timestamp())
				.then(update => {
					if (update.new_feeds != 0) {
						this.websocket.send_get_item_list(this.search_params, this.viewing_category, 0, update.new_feeds)
						.then(resp => {
							console.log('Update Items:', resp);

							this.on_received_update_items(resp.items, resp.notification_ids);
						})
						.catch(e => console.error('Grabbing Feed Items List', e));
					}

					if (update.new_watches != 0) {
						new Notification(`Received ${update.new_watches} new watch update(s).`);

						this.websocket.send_get_watcher_list()
						.then(resp => {
							console.log('Watching:', resp);
							this.watching_listeners = resp.items;
						})
						.catch(e => console.error('Grabbing Watcher List', e));
					}
				})
				.catch(e => console.error('Getting Newest Updates', e));
			})()
			.catch(console.error);
		}, 1000 * 30);
	}

	async reset_feeds() {
		this.feed_items = [];

		let feed_item_list_resp = await this.websocket.send_get_item_list(this.search_params, this.viewing_category, this.page_index * this.page_size, this.page_size);

		this.add_or_update_feed_items(feed_item_list_resp.items, feed_item_list_resp.notification_ids);

		console.log(feed_item_list_resp);
	}

	add_or_update_feed_items(feed_items: ModelItem[], notification_ids: number[]): FeedItem[] {
		let alerting_items: FeedItem[] = [];

		feed_items.map(v => new FeedItem(v, notification_ids.includes(v.id!)))
		.forEach(item => {
			let index_of = this.feed_items.findIndex(i => i.id == item.id);

			if (index_of == -1) {
				this.feed_items.push(item);
			}

			if (item.alert) {
				alerting_items.push(item);
			}
		});

		this.feed_items.sort((a, b) => b.date_added - a.date_added);

		return alerting_items;
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


	//

	on_received_update_items(items: ModelItem[], notification_ids: number[]) {
		// Used for notifications when receiving new items from an update.

		let alerting_items = this.add_or_update_feed_items(items, notification_ids);

		if (alerting_items.length != 0 && this.has_notification_perms()) {
			new Notification(`Received ${alerting_items.length} new items.`);
		}
	}

	get_newest_timestamp(): number {
		let timestamp = 0;

		this.feed_items.forEach(f => { if (f.date > timestamp) { timestamp = f.date; } });

		this.watching_listeners.forEach(f => {
			if (f[1] != null && f[1].date_added > timestamp) { timestamp = f[1].date_added; }
		});

		return timestamp;
	}

	has_notification_perms(): boolean {
		return Notification.permission == 'granted';
	}
}