import { Injectable, Output } from '@angular/core';
import { WebsocketService } from './websocket.service';

@Injectable({
	providedIn: 'root'
})

export class BackgroundService {
	@Output()
	public feed_list: ModelListener[] = [];

	@Output()
	public feed_items: ModelItem[] = [];

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

		// if (core.view != null && core.view instanceof FeedItemsView) {
		// 	core.view.table.reset();
		// }

		let feed_list_resp = await this.websocket.send_get_feed_list();

		this.feed_list = feed_list_resp.items;
		console.log('Feeds:', this.feed_list);

		let viewing_category = null;
		// Set the viewing_category only if viewing table. Otherwise get all.
		// if (core.view != null && core.view instanceof FeedItemsView) {
		// 	viewing_category = core.view.table.viewing_category;
		// }

		let feed_item_list_resp = await this.websocket.send_get_item_list(viewing_category, undefined, undefined);

		let feed_items = feed_item_list_resp.items;//.map(i => new FeedItem(i, feed_item_list_resp.notification_ids.includes(i.id!)));
		feed_items = this.add_or_update_feed_items(feed_items);

		// if (core.view != null && core.view instanceof FeedItemsView) {
		// 	core.view.table.new_items(feed_item_list_resp, feed_items);
		// }

		console.log(feed_item_list_resp);

		let watcher_list_resp = await this.websocket.send_get_watcher_list();
		this.watching_listeners = watcher_list_resp.items;

		return Promise.resolve();
	}

	public init(): void {
		Notification.requestPermission();

		this.init_feeds();

		// TODO: Remove. Actually utilize websockets.
		setInterval(async () => {
			this.websocket.send_get_updates_since(this.get_newest_timestamp())
			.then(update => {
				if (update.new_feeds != 0) {
					this.websocket.send_get_item_list(null, 0, update.new_feeds)
					.then(resp => {
						console.log('Update Items:', resp);

						// this.on_received_update_items(resp.items, resp.notification_ids);

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

						// if (core.view != null && core.view instanceof WatchItemsView) {
						// 	core.view.table.add_sort_render_rows();
						// }
					})
					.catch(e => console.error('Grabbing Watcher List', e));
				}
			})
			.catch(e => console.error('Getting Newest Updates', e));
		}, 1000 * 30);
	}

	add_or_update_feed_items(feed_items: ModelItem[]): ModelItem[] {
		let updated: ModelItem[] = [];

		feed_items.forEach(item => {
			let index_of = this.feed_items.findIndex(i => i.id == item.id);

			if (index_of == -1) {
				this.feed_items.push(item);
				updated.push(item);
			} else {
				updated.push(this.feed_items[index_of]);
			}
		});

		return updated;
	}

	get_feed_by_id(id: number): Optional<ModelListener> {
		return this.feed_list.find(f => f.id == id);
	}

	get_feed_by_url(url: string): Optional<ModelListener> {
		return this.feed_list.find(i => i.url == url);
	}

	on_received_update_items(items: ModelItem[], notification_ids: number[]) {
		// Used for notifications when receiving new items from an update.

		let new_items = this.add_or_update_feed_items(items);

		// Send items to table.
		// if (core.view != null && core.view instanceof FeedItemsView) {
		// 	// TODO: Implement into table func.
		// 	core.view.table.last_req_amount += new_items.length;
		// 	core.view.table.last_total_items += new_items.length;

		// 	core.view.table.add_sort_render_rows(new_items);
		// }

		// let alertable = new_items.filter(i => i.alert);

		// if (alertable.length != 0 && this.has_notification_perms()) {
		// 	new Notification(`Received ${alertable.length} new items.`);
		// }
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