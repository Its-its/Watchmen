import { Injectable, Output } from '@angular/core';
import { FeedItem } from './item/feed-item';
import { FeedListener } from './item/feed-listener';
import { WebsocketService } from './websocket.service';

@Injectable({
	providedIn: 'root'
})

export class BackgroundService {
	@Output()
	public feed_list: FeedListener[] = [];

	@Output()
	public feed_items: FeedItem[] = [];

	public filter_list: FilterGroupListener[] = [];

	watching_listeners = <[ModelWatcher, ModelWatchHistory][]>[];

	constructor(private websocket: WebsocketService) {
		console.log('Created Background Service');

		// this.websocket.send_get_feed_list()
		// 	.then(v => this.feed_list = v.items)
		// 	.catch(console.error);
	}

	// Initial call when loading website.
	private async init_feeds() {
		console.log('Refresh Feeds');

		let feed_list_resp = await this.websocket.send_get_feed_list();

		this.feed_list = feed_list_resp.items.map(v => new FeedListener(v));
		console.log('Feeds:', this.feed_list);

		let viewing_category = null;
		// Set the viewing_category only if viewing table. Otherwise get all.
		// if (core.view != null && core.view instanceof FeedItemsView) {
		// 	viewing_category = core.view.table.viewing_category;
		// }

		this.filter_list = (await this.websocket.send_get_filter_list()).items;

		let feed_item_list_resp = await this.websocket.send_get_item_list(viewing_category, undefined, undefined);

		this.add_or_update_feed_items(feed_item_list_resp.items, feed_item_list_resp.notification_ids);

		console.log(feed_item_list_resp);

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
						this.websocket.send_get_item_list(null, 0, update.new_feeds)
						.then(resp => {
							console.log('Update Items:', resp);

							this.on_received_update_items(resp.items, resp.notification_ids);

							// if (core.view != null && core.view instanceof FeedItemsView) {
							// 	if (core.view.table.container.scrollTop < 40 * 4) {
							// 		core.view.table.container.scrollTo({ top: 0, behavior: 'smooth' });
							// 	}
							// }
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