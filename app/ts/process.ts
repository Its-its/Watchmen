// Background processing.

import core from './core';
import FeedItemsView from './views/feed/items';
import WatchItemsView from './views/watch/items';

import {
	send_get_item_list,
	send_get_feed_list,
	send_get_updates_since,
	send_get_watcher_list
} from './socket';


export class FeedListener {
	id: number;

	date_added: number;
	ignore_if_not_new: boolean;
	global_show: boolean;
	last_called: number;
	remove_after: number;
	sec_interval: number;
	url: string;
	title: string;
	description: string;
	generator: string;

	constructor(opts: ModelListener) {
		this.id = opts.id!;
		this.url = opts.url;
		this.title = opts.title;
		this.description = opts.description;
		this.generator = opts.generator;
		this.global_show = opts.global_show;
		this.ignore_if_not_new = opts.ignore_if_not_new;
		this.remove_after = opts.remove_after;
		this.sec_interval = opts.sec_interval;
		this.date_added = opts.date_added;
		this.last_called = opts.last_called;
	}
}

export class FeedItem {
	id: number;

	guid: string;
	title: string;
	author: string;
	content: string;
	link: string;
	date: number;
	hash: string;

	date_added: number;
	is_read: boolean;
	is_starred: boolean;
	is_removed: boolean;
	tags: string;
	feed_id: number;

	alert: boolean;

	constructor(opts: ModelItem, alert: boolean) {
		this.id = opts.id!;
		this.guid = opts.guid;
		this.title = opts.title;
		this.author = opts.author;
		this.content = opts.content;
		this.link = opts.link;
		this.date = opts.date;
		this.hash = opts.hash;
		this.date_added = opts.date_added;
		this.is_read = opts.is_read;
		this.is_starred = opts.is_starred;
		this.is_removed = opts.is_removed;
		this.tags = opts.tags;
		this.feed_id = opts.feed_id;
		this.alert = alert;
	}

	public parse_timestamp(): string {
		let date = this.date * 1000;

		if (date + (1000 * 60 * 60 * 24) > Date.now()) {
			return elapsed_to_time_ago(Date.now() - date);
		} else {
			return new Date(date).toLocaleString()
		}
	}
}


export default class BackgroundProcess {
	feed_listeners = <FeedListener[]>[];
	feed_items = <FeedItem[]>[];

	watching_listeners = <[ModelWatcher, ModelWatchHistory][]>[];

	constructor() {
		//
	}

	register_updates() {
		Notification.requestPermission();

		setInterval(() => {
			if (core.view != null && core.view instanceof FeedItemsView) {
				core.view.table.rows.forEach(r => r.update_date_element());
			} else if (core.view != null && core.view instanceof WatchItemsView) {
				core.view.table.rows.forEach(r => r.update_date_element());
			}

			send_get_updates_since(this.get_newest_timestamp(), (_, update) => {
				if (update == null) return;

				if (update.new_feeds != 0) {
					send_get_item_list(null, 0, update.new_feeds, (_, resp) => {
						console.log('Update Items:', resp);

						this.on_received_update_items(resp!.items, resp!.notification_ids);

						if (core.view != null && core.view instanceof FeedItemsView) {
							if (core.view.table.container.scrollTop < 40 * 4) {
								core.view.table.container.scrollTo({ top: 0, behavior: 'smooth' });
							}
						}
					});
				}

				if (update.new_watches != 0) {
					send_get_watcher_list((_, resp) => {
						this.watching_listeners = resp!.items;
						console.log('Watching:', resp);

						if (core.view != null && core.view instanceof WatchItemsView) {
							core.view.table.add_sort_render_rows();
						}
					});
				}
			});
		}, 1000 * 30);
	}

	// Initial call when loading website.
	refresh_feeds(finished?: () => any) {
		console.log('Refresh Feeds');

		if (core.view != null && core.view instanceof FeedItemsView) {
			core.view.table.reset();
		}

		send_get_feed_list((_, feed_opts) => {
			this.feed_listeners = feed_opts!.items.map(i => new FeedListener(i));
			console.log('Feeds:', this.feed_listeners);

			let viewing_category = null;
			// Set the viewing_category only if viewing table. Otherwise get all.
			if (core.view != null && core.view instanceof FeedItemsView) {
				viewing_category = core.view.table.viewing_category;
			}

			send_get_item_list(viewing_category, undefined, undefined, (_, items) => {
				let feed_items = items!.items.map(i => new FeedItem(i, items!.notification_ids.includes(i.id!)));
				feed_items = this.add_or_update_feed_items(feed_items);

				if (core.view != null && core.view instanceof FeedItemsView) {
					core.view.table.new_items(items!, feed_items);
				}

				console.log(items);

				if (finished != null) finished();
			});
		});

		send_get_watcher_list((_, resp) => this.watching_listeners = resp!.items);
	}

	add_or_update_feed_items(feed_items: FeedItem[]): FeedItem[] {
		let updated: FeedItem[] = [];

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

	get_feed_by_id(id: number): Optional<FeedListener> {
		return this.feed_listeners.find(f => f.id == id);
	}

	get_feed_by_url(url: string): Optional<FeedListener> {
		return this.feed_listeners.find(i => i.url == url);
	}

	on_received_update_items(items: ModelItem[], notification_ids: number[]) {
		// Used for notifications when receiving new items from an update.

		let new_items = this.add_or_update_feed_items(items.map(i => new FeedItem(i, notification_ids.includes(i.id!))));

		// Send items to table.
		if (core.view != null && core.view instanceof FeedItemsView) {
			// TODO: Implement into table func.
			core.view.table.last_req_amount += new_items.length;
			core.view.table.last_total_items += new_items.length;

			core.view.table.add_sort_render_rows(new_items);
		}

		let alertable = new_items.filter(i => i.alert);

		if (alertable.length != 0 && this.has_notification_perms()) {
			new Notification(`Received ${alertable.length} new items.`)
		}
	}

	get_newest_timestamp(): number {
		let timestamp = 0;

		for (let i = 0; i < this.feed_items.length; i++) {
			let row = this.feed_items[i];

			if (row.date > timestamp) {
				timestamp = row.date;
			}
		}

		return timestamp;
	}

	has_notification_perms(): boolean {
		return Notification.permission == 'granted';
	}
}

export function elapsed_to_time_ago(elapsed: number): string {
	let msPerMinute = 60 * 1000;
	let msPerHour = msPerMinute * 60;

	if (elapsed < msPerMinute) {
		return Math.floor(elapsed/1000) + 's ago';
	}

	if (elapsed < msPerHour) {
		return Math.floor(elapsed/msPerMinute) + 'm ago';
	}

	return `${Math.floor(elapsed/msPerHour)}h, ${Math.floor(elapsed/msPerMinute) % 60}m ago`;
}
