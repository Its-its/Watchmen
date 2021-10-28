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

function elapsed_to_time_ago(elapsed: number): string {
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
