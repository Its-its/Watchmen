export class FeedListener {
	id: number;

	enabled: boolean;

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
		this.enabled = opts.enabled;
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
