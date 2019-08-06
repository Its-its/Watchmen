type Obj<I> = { [name: string]: I };

type Nullable<I> = I | null;

interface SocketResponse {
	[name: string]: any;

	message_id?: number;
	error?: string;
	result?: {
		method: string;
		params: { [name: string]: any; };
	};
}

type ResponseFunc<V> = (error: any, value: V, method: string) => any;

interface AwaitingReponse {
	sent: number,
	timeout_seconds: number,

	msg_id: number,
	resp_func?: ResponseFunc<any>
}

// Models
interface ModelCategory {
	id?: number;

	date_added: number;
	name: string;
	name_lowercase: string;
	position: number;
};

interface ModelListener {
	id?: number;

	title: string;
	url: string;
	description: string;
	date_added: number;
	generator: string;
	ignore_if_not_new: boolean;
	global_show: boolean;
	last_called: number;
	remove_after: number;
	sec_interval: number;
}

interface ModelEditListener {
	title?: string;
	description?: string;
	generator?: string;

	ignore_if_not_new?: boolean;
	global_show?: boolean;

	remove_after?: number;
	sec_interval?: number;
}

interface ModelItem {
	id?: number;

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
}

interface ModelFeedCategory {
	id?: number;

	feed_id: number;
	category_id: number;
}

// Responses

interface RemoveListenerResponse {
	//
}

interface EditListenerResponse {
	affected: number;
	listener: ModelEditListener;
}

interface ItemListResponse {
	item_count: number;
	skip_count: number;
	total_items: number;
	items: ModelItem[];
};

interface FeedListResponse {
	items: ModelListener[];
};

interface CreateListenerResponse {
	affected: number;
	listener: ModelListener;
}

interface UpdatesResponse {
	new_items: number;
	since: number;
};

interface CreateCategoryResponse {
	affected: number;
	category: ModelCategory;
};

interface CategoryListResponse {
	categories: ModelCategory[];
	category_feeds: ModelFeedCategory[];
}

interface AddCategoryFeedResponse {
	affected: number;
	category: ModelFeedCategory
}


class SocketManager {
	socket = new WebSocket("ws://" + window.location.host + "/ws/");

	last_message_id = 0;

	awaiting_response: AwaitingReponse[] = [];

	constructor() {
		this.socket.onmessage = event => this.onMessage(JSON.parse(event.data));
		this.socket.onopen = event => this.on_open(event);
	}


	public on_open(ev: Event): any {
		app.on_connection_open();
	}

	public onMessage(resp: SocketResponse) {
		if (resp.error != null) {
			return console.error(resp);
		}

		if (resp.result != null) {
			var result = resp.result;
			// console.log('Received:', result);

			if (resp.message_id != null) {
				return this.update_response(resp);
			}

			// switch(result.method) {
			// 	default: {
					console.log('Default:', result);
			// 	}
			// }
		}
	}


	public next_msg_id(): number {
		return this.last_message_id++;
	}

	public update_response(value: SocketResponse) {
		for (var i = 0; i < this.awaiting_response.length; i++) {
			var resp = this.awaiting_response[i];

			if (resp.msg_id == value.message_id) {
				if (resp.resp_func) {
					resp.resp_func(
						value.error,
						value.result!.params,
						value.result!.method
					);
				}
				this.awaiting_response.splice(i, 1);
				break;
			}
		}
	}

	//

	public send_response(name: string, opts: Obj<any>, response: ResponseFunc<any>) {
		var message_id = this.next_msg_id();

		this.awaiting_response.push({
			msg_id: message_id,
			resp_func: response,
			sent: Date.now(),
			timeout_seconds: 60 * 5
		});

		var method = {
			"method": name,
			"params": opts
		};

		var wrapper = {
			"method": "frontend",
			"params": {
				"message_id": message_id,
				"command": method
			}
		};

		// console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper));
	}

	public send_notification(name: string, opts?: Obj<any>) {
		var method = {
			"method": name,
			"params": opts
		};

		var wrapper = {
			"method": "frontend",
			"params": {
				"command": method
			}
		};

		// console.log('Sending:', wrapper);

		this.socket.send(JSON.stringify(wrapper));
	}
}

class Table {
	container = document.createElement('div');

	row_ids: number[] = [];
	rows: TableItem[] = [];

	// class for infinite-scroll or page buttons

	// filters = [];

	current_section = -1;

	// Set from get_items
	last_req_amount = 0;
	last_skip_amount = 0;
	last_total_items = 0;

	waiting_for_more_feeds = false;

	constructor() {
		this.container.className = 'feed-container';
		this.init();
	}

	public reset() {
		while (this.container.firstChild != null) {
			this.container.removeChild(this.container.firstChild);
		}

		this.last_req_amount = 0;
		this.last_skip_amount = 0;
		this.last_total_items = 0;
		this.waiting_for_more_feeds = false;
		this.current_section = -1;
		this.row_ids = [];
		this.rows = [];
	}

	public init() {
		this.container.addEventListener('scroll', e => {
			// Bottom of page
			if (
				this.container.scrollTop
				> this.container.scrollHeight
					- window.innerHeight
					- (TableItem.HEIGHT * 6)
				) {
				this.grab_new_rows(false);
			}
		});
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild != null) {
			this.container.removeChild(this.container.firstChild);
		}

		this.render_rows(this.rows);

		return this.container;
	}

	public render_rows(rows: TableItem[]) {
		var section_names = [
			'Today',
			'Yesterday',
			'This Week',
			'This Month',
			'Last Month',
			'This Year',
			'Last Year'
		];

		rows.forEach(r => {
			var section = get_section_from_date(r.date * 1000);
			if (section != this.current_section) {
				this.current_section = section;

				var section_name = section_names[section];

				var section_html = document.createElement('div');
				section_html.className = 'section ' + section_name.toLowerCase().replace(' ', '-');
				section_html.innerHTML = `<span>${section_name}</span>`;

				this.container.appendChild(section_html);
			}

			this.container.appendChild(r.render());
		});

		function get_section_from_date(timestamp: number): number {
			var ct = Date.now();

			// Last Year
			if (timestamp < ct - (1000 * 60 * 60 * 24 * 365 * 2)) return 6;

			// This Year
			if (timestamp < ct - (1000 * 60 * 60 * 24 * 365 * 2)) return 5;

			// Last Month
			if (timestamp < ct - (1000 * 60 * 60 * 24 * 30 * 2)) return 4;

			// This Month
			if (timestamp < ct - (1000 * 60 * 60 * 24 * 30)) return 3;

			// This Week
			if (timestamp < ct - (1000 * 60 * 60 * 24 * 7)) return 2;

			// Yesterday
			if (timestamp < ct - (1000 * 60 * 60 * 24)) return 1;

			return 0;
		}
	}


	public new_items(opts: ItemListResponse) {
		this.last_skip_amount = opts.skip_count;
		this.last_req_amount = opts.item_count;
		this.last_total_items = opts.total_items;

		this.add_sort_render_rows(opts.items);
	}

	public add_sort_render_rows(items: any[]) {
		var new_items = items
		.filter(i => this.row_ids.indexOf(i.id) == -1)
		.map(i => {
			this.row_ids.push(i.id);
			return new TableItem(i);
		});

		this.rows = this.rows.concat(new_items);

		this.rows.sort(this.sort_item_func('date', 1));
		this.row_ids.sort();

		this.render();

		return new_items;
	}

	public grab_new_rows(going_backwards: boolean) {
		this.waiting_for_more_feeds = true;

		// Scrolling Upwards
		if (going_backwards) {}

		var skip_amount = this.last_skip_amount + this.last_req_amount;

		// If next request is going to be outside of the the total items we have, return.
		if (skip_amount > this.last_total_items) {
			this.waiting_for_more_feeds = true;
			return;
		}

		console.log('Skipping:', skip_amount + '/' + this.last_total_items);

		send_get_item_list(app.viewing_category, skip_amount, 25, (err, items: ItemListResponse) => {
			if (err != null) {
				this.waiting_for_more_feeds = false;
				console.error(err);
				return;
			}

			this.new_items(items);

			this.waiting_for_more_feeds = false;
		});
	}

	public remove_items_by_id(item_id: number[] | number) {
		if (Array.isArray(item_id)) {
			item_id.forEach(i => this.remove_items_by_id(i));
		} else {
			var index = this.row_ids.indexOf(item_id);
			if (index != -1) {
				this.row_ids.splice(index, 1);

				for (var i = 0; i < this.rows.length; i++) {
					if (this.rows[i].id == item_id) {
						this.rows.splice(i, 1);
						break;
					}
				}
			}
		}
	}

	public sort_item_func(sort_method: 'id' | 'date_added' | 'date', sort_order: 1 | -1): (a: TableItem, b: TableItem) => number {
		return (a, b) => (b[sort_method] - a[sort_method]) * sort_order;
	}

	public can_continue_scroll_up(): boolean {
		return this.last_skip_amount != 0;
	}

	public can_continue_scroll_down(): boolean {
		return this.last_skip_amount  + this.last_req_amount < this.last_total_items;
	}

	public get_newest_timestamp(): number {
		var timestamp = 0;

		for (var i = 0; i < this.rows.length; i++) {
			var row = this.rows[i];

			if (row.date > timestamp) {
				timestamp = row.date;
			}
		}

		return timestamp;
	}
}

class TableItem {
	container = document.createElement('div');

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

	// Element
	date_element = document.createElement('span');

	constructor(opts: ModelItem) {
		this.container.className = 'feed-item';

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
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild) {
			this.container.removeChild(this.container.firstChild);
		}

		var list = document.createElement('ul');
		list.className = 'list horizontal';
		this.container.appendChild(list);

		// feed site name
		list.appendChild((() => {
			var li = document.createElement('li');
			li.className = 'list-item feed-name';

			var feed = app.get_feed_by_id(this.feed_id)!;

			var span = document.createElement('a');
			span.innerText = feed.generator || feed.title;
			span.title = span.innerText;
			span.href = `/?feed=${this.feed_id}`;
			li.appendChild(span);

			return li;
		})());

		// Title
		list.appendChild((() => {
			var li = document.createElement('li');
			li.className = 'list-item title';

			var span = document.createElement('a');
			span.className = 'default';
			span.innerText = this.title;
			span.title = span.innerText;
			span.href = this.link;

			span.addEventListener('click', e => {
				console.log('Showing item info');
				e.preventDefault();
				return false;
			});

			li.appendChild(span);

			return li;
		})());

		// Date
		list.appendChild((() => {
			var li = document.createElement('li');
			li.className = 'list-item date';

			this.update_date_element();
			this.date_element.title = new Date(this.date * 1000).toLocaleString();
			li.appendChild(this.date_element);

			return li;
		})());

		// site link
		list.appendChild((() => {
			var li = document.createElement('li');
			li.className = 'list-item link';

			var a_href = document.createElement('a');
			a_href.className = 'default';
			a_href.innerText = 'link';
			a_href.href = this.link;
			li.appendChild(a_href);

			return li;
		})());

		return this.container;
	}


	public parse_timestamp(): string {
		var date = this.date * 1000;

		if (date + (1000 * 60 * 60 * 24) > Date.now()) {
			return elapsed_to_time_ago(Date.now() - date);
		} else {
			return new Date(date).toLocaleString()
		}
	}

	public update_date_element() {
		this.date_element.innerText = this.parse_timestamp();
	}

	static HEIGHT = 41;
}

class Listener {
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

// Make button for categories to see filtered or all for said category. (All category shows all filtered)
class SidebarItem {
	id: number;
	name: string;
	name_lowercase: string;
	date_added: number;
	position: number;

	category_feeds: ModelFeedCategory[];

	constructor(opts: ModelCategory, category_feeds?: ModelFeedCategory[]) {
		this.id = opts.id!;
		this.name = opts.name;
		this.name_lowercase = opts.name_lowercase;
		this.date_added = opts.date_added;
		this.position = opts.position;

		this.category_feeds = category_feeds || [];
	}

	public render(): HTMLLIElement {
		var container = document.createElement('li');
		container.className = 'tree-item' + (
			(app.viewing_category == null && this.id == -1)
			|| app.viewing_category == this.id ? ' active' : '');

		// Options
		if (this.id != -1) {
			var options_container = document.createElement('div');
			options_container.className = 'tree-item-options';
			options_container.innerText = 'O';
			container.appendChild(options_container);

			options_container.addEventListener('click', _ => this.show_editor());
		}

		// Title

		var title_container = document.createElement('a');
		title_container.className = 'tree-item-title';
		title_container.href = '/' + (this.id == -1 ? '' : `?cat=${this.id}`);
		container.appendChild(title_container);

		var title = document.createElement('span');
		title.className = 'title';
		title.innerText = this.name;
		title_container.appendChild(title);

		return container;
	}

	public show_editor() {
		create_popup((container, open, close) => {
			var form = document.createElement('div');
			form.className = 'form-group';
			container.appendChild(form);

			// Category Text
			var cat_row = document.createElement('div');
			cat_row.className = 'form-row';
			form.appendChild(cat_row);

			let inputs: [HTMLInputElement, number][] = [];

			app.feeds.forEach(feed => {
				let cont = document.createElement('div');

				let input = document.createElement('input');
				input.checked = this.contains_cat_feed_by_feed_id(feed.id);
				input.type = 'checkbox';
				input.name = 'feed-' + feed.id;
				cont.appendChild(input);

				let label = document.createElement('label');
				(<any>label).for = 'feed-' + feed.id;
				label.innerText = feed.title;
				cont.appendChild(label);

				// Generator
				var generator = document.createElement('div');
				generator.innerText = feed.generator.slice(0, 20) + (feed.generator.length > 20 ? '..' : '');
				generator.title = feed.generator;
				cont.appendChild(generator);

				// Description
				var descr = document.createElement('pre');
				descr.innerText = feed.description.slice(0, 80) + (feed.description.length > 80 ? '..' : '');
				descr.title = feed.description;
				cont.appendChild(descr);

				cat_row.appendChild(cont);

				inputs.push([input, feed.id]);
			});

			// Submit
			var sub_row = document.createElement('div');
			sub_row.className = 'form-row';
			form.appendChild(sub_row);

			var submit = document.createElement('div');
			submit.className = 'button';
			submit.innerText = 'Create';
			sub_row.appendChild(submit);

			submit.addEventListener('click', _ => {
				var to_send_adding: number[] = [];
				var to_send_removing: number[] = [];

				inputs.forEach(input_id => {
					var input = input_id[0],
						feed_id = input_id[1];

					var contains_feed = this.contains_cat_feed_by_feed_id(feed_id);

					// Adding Feed
					if (!contains_feed && input.checked) {
						to_send_adding.push(feed_id);
					}

					// Removing Feed
					else if (contains_feed && !input.checked) {
						var feed = this.get_cat_feed_by_feed_id(feed_id);
						if (feed == null) return console.log('No category feed: ' + feed_id + ',', this.category_feeds);

						to_send_removing.push(feed.id!);
					}
				});

				var other_finished = false;

				if (to_send_adding.length != 0) {
					for_each(to_send_adding, (feed_id, fin) => {
						send_add_feed_to_category(feed_id, this.id, fin);
					}, _ => {
						if (other_finished) {
							app.refresh_categories();

							if (app.viewing_category == this.id) {
								app.refresh_feeds();
							}
						}

						other_finished = true;
					});
				}

				if (to_send_removing.length != 0) {
					for_each(to_send_removing, (feed_id, fin) => {
						send_remove_feed_from_category(feed_id, fin);
					}, _ => {
						if (other_finished) {
							app.refresh_categories();

							if (app.viewing_category == this.id) {
								app.refresh_feeds();
							}
						}

						other_finished = true;
					});
				}

				close();
			});

			open();
		});
	}

	public get_cat_feed_by_id(id: number): Nullable<ModelFeedCategory> {
		if (this.category_feeds != null) {
			for (var i = 0; i < this.category_feeds.length; i++) {
				var element = this.category_feeds[i];
				if (element.id == id) return element;
			}
		}

		return null;
	}

	public get_cat_feed_by_feed_id(id: number): Nullable<ModelFeedCategory> {
		if (this.category_feeds != null) {
			for (var i = 0; i < this.category_feeds.length; i++) {
				var element = this.category_feeds[i];
				if (element.feed_id == id) return element;
			}
		}

		return null;
	}

	public contains_cat_feed_by_id(id: number): boolean {
		return this.get_cat_feed_by_id(id) != null;
	}

	public contains_cat_feed_by_feed_id(id: number): boolean {
		return this.get_cat_feed_by_feed_id(id) != null;
	}

	public remove_feed(id: number) {
		if (this.category_feeds != null) {
			for (var i = 0; i < this.category_feeds.length; i++) {
				var element = this.category_feeds[i];
				if (element.id == id) {
					this.category_feeds.splice(i, 1);

					break;
				}
			}
		}
	}
}

const app = {
	viewing_category: <Nullable<number>>null,
	viewing_feed: <Nullable<number>>null,

	feeds: <Listener[]>[],
	categories: <SidebarItem[]>[],

	table: new Table(),

	socket: new SocketManager(),

	// Initiation

	init() {
		this.init_top_nav();

		document.getElementById('container')!.appendChild(this.table.render());

		const url_params = new URLSearchParams(location.search.slice(1));

		var cat_str = url_params.get('cat');
		this.viewing_category = cat_str ? parseInt(cat_str) : null;

		var feed_str = url_params.get('feed');
		this.viewing_feed = feed_str ? parseInt(feed_str) : null;

		if (this.viewing_category != null && this.viewing_feed != null) {
			window.location.pathname = '/?cat=' + this.viewing_category;
		}
	},

	init_top_nav() {
		var top_nav = document.getElementById('top-nav-bar')!;

		var nav_left = document.createElement('div');
		nav_left.className = 'nav-container left';
		top_nav.appendChild(nav_left);

		var new_listener = document.createElement('div');
		new_listener.className = 'button';
		new_listener.innerText = 'New Feed';
		nav_left.appendChild(new_listener);

		new_listener.addEventListener('click', _ => {
			create_popup((container, open, close) => {
				var form = document.createElement('div');
				form.className = 'form-group';
				container.appendChild(form);

				// Feed URL
				var cat_row = document.createElement('div');
				cat_row.className = 'form-row';
				form.appendChild(cat_row);

				var cat_text = document.createElement('input');
				cat_text.placeholder = 'Feed URL';
				cat_text.type = 'text';
				cat_row.appendChild(cat_text);

				// Submit
				var sub_row = document.createElement('div');
				sub_row.className = 'form-row';
				form.appendChild(sub_row);

				var submit = document.createElement('div');
				submit.className = 'button';
				submit.innerText = 'Create';
				sub_row.appendChild(submit);

				submit.addEventListener('click', _ => {
					send_create_listener(cat_text.value, (err, opts) => {
						if (err != null) {
							return console.error('create_listener: ', err);
						}

						console.log('create_listener:', opts);

						if (opts.affected != 0) {
							// We only use the url of opts.listener. Since it doesn't include the id of it.

							app.refresh_feeds(() => {
								close();

								var feed: Nullable<Listener> = null;

								if (feed = app.get_feed_by_url(opts.listener.url)) {
									app.edit_listener(feed);
								} else {
									console.log('Unable to find feed to edit from url.');
								}
							});
						}
					});

				});

				open();
			});
		});


		var nav_right = document.createElement('div');
		nav_right.className = 'nav-container right';
		top_nav.appendChild(nav_right);

		// var show_listeners = document.createElement('div');
		// show_listeners.className = 'button';
		// show_listeners.innerText = 'Listeners';
		// nav_right.appendChild(show_listeners);

		// show_listeners.addEventListener('click', _ => {
		// 	create_popup((container, open, close) => {
		// 		//

		// 		open();
		// 	});
		// });
	},

	// TODO: Something with the Remove is not working properly.
	edit_listener(listener: Listener) {
		create_popup((container, open, close) => {
			var form = document.createElement('div');
			form.className = 'form-group';
			container.appendChild(form);

			// URL
			var url_row = document.createElement('div');
			url_row.className = 'form-row';
			form.appendChild(url_row);

			var url_input = document.createElement('input');
			url_input.placeholder = '((Unknown Feed URL))';
			url_input.value = listener.url;
			url_input.type = 'text';
			url_input.disabled = true;
			url_row.appendChild(url_input);

			// Generator
			var gen_row = document.createElement('div');
			gen_row.className = 'form-row';
			form.appendChild(gen_row);

			var gen_input = document.createElement('input');
			gen_input.disabled = true;
			gen_input.placeholder = 'Generator';
			gen_input.value = listener.generator;
			gen_input.type = 'text';
			gen_row.appendChild(gen_input);

			// Title
			var title_row = document.createElement('div');
			title_row.className = 'form-row';
			form.appendChild(title_row);

			var title_input = document.createElement('input');
			title_input.placeholder = 'No title. Please specify one.';
			title_input.value = listener.title;
			title_input.type = 'text';
			title_row.appendChild(title_input);

			// Description
			var desc_row = document.createElement('div');
			desc_row.className = 'form-row';
			form.appendChild(desc_row);

			var desc_input = document.createElement('textarea');
			desc_input.placeholder = 'Listener Description';
			desc_input.value = listener.description;
			desc_row.appendChild(desc_input);


			// Edit
			var sub_row = document.createElement('div');
			sub_row.className = 'form-row';
			form.appendChild(sub_row);

			var submit = document.createElement('div');
			submit.className = 'button';
			submit.innerText = 'Edit';
			sub_row.appendChild(submit);

			// Delete
			var remove = document.createElement('div');
			remove.className = 'button';
			remove.innerText = 'Delete';
			sub_row.appendChild(remove);

			var to_check: ['title' | 'description', HTMLInputElement | HTMLTextAreaElement][] = [
				[ 'title', title_input ],
				[ 'description', desc_input ]
			];

			submit.addEventListener('click', () => {
				var edited: ModelEditListener = {};

				to_check.forEach(i => {
					var name = i[0], input = i[1];
					if (listener[name] != input.value) edited[name] = input.value;
				});

				if (Object.keys(edited).length != 0) {
					send_edit_listener(listener.id, edited, (e, opts) => {
						if (e != null) return console.error('edit_listener:', e);

						// TODO: if (opts.affected == 0) {}

						for(var name in edited) {
							//@ts-ignore
							listener[name] = edited[name];
						}

						// Re-render table.
						app.table.render();
					});
				}

				close();
			});

			remove.addEventListener('click', () => {
				close();

				// Option on what to remove
				create_popup((container, open, close) => {
					// Submit
					var sub_row = document.createElement('div');
					sub_row.className = 'form-row';
					container.appendChild(sub_row);

					var partial = document.createElement('div');
					partial.className = 'button';
					partial.innerText = 'Partial Delete (Only remove feed)';
					sub_row.appendChild(partial);

					// Delete
					var fully = document.createElement('div');
					fully.className = 'button';
					fully.innerText = 'Fully Delete (Remove feed AND feed items)';
					sub_row.appendChild(fully);

					let partial_func = () => remove(false);
					let full_func = () => remove(true)

					partial.addEventListener('click', partial_func);
					fully.addEventListener('click', full_func);

					function remove(full: boolean) {
						fully.removeEventListener('click', full_func);
						partial.removeEventListener('click', partial_func);

						send_remove_listener(listener.id, full, (err, opts) => {
							console.log('send_remove_listener:', err, opts);

							app.refresh_feeds(() => close());
						});
					}

					open();
				});

			});

			open();
		});
	},

	create_nav_items(opts: CategoryListResponse) {
		// New Category Button
		var nav_bar = document.getElementById('nav-bar')!;

		var create_button = document.createElement('div');
		create_button.className = 'button new-category';
		create_button.innerText = 'New Category';
		nav_bar.appendChild(create_button);

		create_button.addEventListener('click', () => {
			create_popup((container, open, close) => {
				var form = document.createElement('div');
				form.className = 'form-group';
				container.appendChild(form);

				// Category Text
				var cat_row = document.createElement('div');
				cat_row.className = 'form-row';
				form.appendChild(cat_row);

				var cat_text = document.createElement('input');
				cat_text.placeholder = 'Category Name';
				cat_text.type = 'text';
				cat_row.appendChild(cat_text);

				// Submit
				var sub_row = document.createElement('div');
				sub_row.className = 'form-row';
				form.appendChild(sub_row);

				var submit = document.createElement('div');
				submit.className = 'button';
				submit.innerText = 'Create';
				sub_row.appendChild(submit);

				submit.addEventListener('click', _ => {
					console.log(cat_text.value);
					console.log('Next Category: ' + app.next_category_id());

					send_create_category(cat_text.value, (_, opts) => {
						send_get_category_list((_, opts) => app.create_categories(opts));
					});

					close();
				});

				open();
			});
		});

		this.create_categories(opts);
	},

	create_categories(opts: CategoryListResponse) {
		var nav_items = document.getElementById('nav-bar-items')!;
		while (nav_items.firstChild) nav_items.removeChild(nav_items.firstChild);

		var opts_items: ModelCategory[] = [
			{ id: -1, name: "All", name_lowercase: "all", date_added: 0, position: -1 }
		];
		opts_items = opts_items.concat(opts.categories);
		opts_items.sort((a, b) => a.position - b.position);

		var items = opts_items.map(i => new SidebarItem(i, opts.category_feeds.filter(f => f.category_id == i.id)));
		items.forEach(i => nav_items.appendChild(i.render()));

		this.categories = items;
	},

	on_connection_open() {
		// Updates
		setInterval(() => {
			app.table.rows
			.forEach(r => r.update_date_element());

			send_get_updates_since(app.table.get_newest_timestamp(), (_, update) => {
				var is_towards_top = app.table.container.scrollTop < 40 * 4;

				if (update.new_items != 0) {
					send_get_item_list(null, 0, update.new_items, (_, resp) => {
						console.log('Items:', resp);

						app.on_received_update_items(resp.items);

						if (is_towards_top) {
							app.table.container.scrollTo({ top: 0, behavior: 'smooth' });
						}
					});
				}
			});
		}, 1000 * 30);

		// Get Current feeds
		this.refresh_feeds();//(() => this.edit_listener(this.feeds[this.feeds.length - 1]));

		// Get Categories (init)
		send_get_category_list((_, opts) => {
			console.log('Categories:', opts);
			this.create_nav_items(opts);
		});
	},

	refresh_categories() {
		send_get_category_list((_, opts) => {
			this.create_categories(opts);
		});
	},

	refresh_feeds(finished?: () => any) {
		app.table.reset();

		send_get_feed_list((_, feed_opts) => {
			this.feeds = feed_opts.items.map(i => new Listener(i));
			console.log('Feeds:', this.feeds);

			send_get_item_list(this.viewing_category, undefined, undefined, (_, items) => {
				app.table.new_items(items);
				console.log('Items:', app.table.rows);

				if (finished != null) finished();
			});
		});
	},

	on_received_update_items(items: ModelItem[]) {
		// Used for notifications when receiving new items from an update.

		// Send items to table.
		var table_items = app.table.add_sort_render_rows(items);

		if (this.has_notification_perms()) {
			new Notification(`Received ${table_items.length} new items.`)
		}
	},

	get_category_by_id(id: number): Nullable<SidebarItem> {
		for (let i = 0; i < this.categories.length; i++) {
			var feed = this.categories[i];

			if (feed.id == id) {
				return feed;
			}
		}

		return null;
	},

	get_feed_by_id(id: number): Nullable<Listener> {
		for (let i = 0; i < this.feeds.length; i++) {
			var feed = this.feeds[i];

			if (feed.id == id) {
				return feed;
			}
		}

		return null;
	},

	get_feed_by_url(url: string): Nullable<Listener> {
		for (let i = 0; i < this.feeds.length; i++) {
			var feed = this.feeds[i];

			if (feed.url == url) {
				return feed;
			}
		}

		return null;
	},

	has_notification_perms(): boolean {
		return Notification.permission == 'granted';
	},

	next_category_id(): number {
		var largest = 0;

		this.categories.forEach(c => c.id > largest ? largest = c.id : null);

		return largest;
	}
};

app.init();

// Sending

// cat
function send_create_category(name: string, cb?: ResponseFunc<CreateCategoryResponse>) {
	var opts = {
		name: name,
		position: app.next_category_id()
	};

	if (cb == null) {
		app.socket.send_notification('add_category', opts);
	} else {
		app.socket.send_response('add_category', opts, cb);
	}
}

function send_get_category_list(cb?: ResponseFunc<CategoryListResponse>) {
	if (cb == null) {
		app.socket.send_notification('category_list');
	} else {
		app.socket.send_response('category_list', {}, cb);
	}
}

function send_add_feed_to_category(feed_id: number, category_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	var opts = {
		feed_id: feed_id,
		category_id: category_id
	};

	if (cb == null) {
		app.socket.send_notification('add_feed_category', opts);
	} else {
		app.socket.send_response('add_feed_category', opts, cb);
	}
}

function send_remove_feed_from_category(cat_feed_id: number, cb?: ResponseFunc<AddCategoryFeedResponse>) {
	var opts = {
		id: cat_feed_id
	};

	if (cb == null) {
		app.socket.send_notification('remove_feed_category', opts);
	} else {
		app.socket.send_response('remove_feed_category', opts, cb);
	}
}


// items
function send_get_item_list(category_id: Nullable<number>, skip?: number, items?: number, cb?: ResponseFunc<ItemListResponse>) {
	var opts = {
		category_id: category_id,
		items: items,
		skip: skip
	};

	if (cb == null) {
		app.socket.send_notification('item_list', opts);
	} else {
		app.socket.send_response('item_list', opts, cb);
	}
}

// listeners
function send_get_feed_list(cb?: ResponseFunc<FeedListResponse>) {
	if (cb == null) {
		app.socket.send_notification('feed_list');
	} else {
		app.socket.send_response('feed_list', {}, cb);
	}
}

function send_create_listener(url: string, cb?: ResponseFunc<CreateListenerResponse>) {
	var opts = {
		url: url
	};

	if (cb == null) {
		app.socket.send_notification('add_listener', opts);
	} else {
		app.socket.send_response('add_listener', opts, cb);
	}
}

function send_edit_listener(id: number, editing: ModelEditListener,  cb?: ResponseFunc<EditListenerResponse>) {
	var opts = {
		id: id,
		editing: editing
	};

	if (cb == null) {
		app.socket.send_notification('edit_listener', opts);
	} else {
		app.socket.send_response('edit_listener', opts, cb);
	}
}

function send_remove_listener(id: number, rem_stored: boolean,  cb?: ResponseFunc<RemoveListenerResponse>) {
	var opts = {
		id: id,
		rem_stored: rem_stored
	};

	if (cb == null) {
		app.socket.send_notification('remove_listener', opts);
	} else {
		app.socket.send_response('remove_listener', opts, cb);
	}
}


// other
function send_get_updates_since(since_timestamp: number, cb?: ResponseFunc<UpdatesResponse>) {
	// Gets the amount of feeds that are newer than feed_timestamp.
	var opts = {
		since: since_timestamp
	};

	if (cb == null) {
		app.socket.send_notification('updates', opts);
	} else {
		app.socket.send_response('updates', opts, cb);
	}
}

// Utils
function elapsed_to_time_ago(elapsed: number): string {
	var msPerMinute = 60 * 1000;
	var msPerHour = msPerMinute * 60;

	if (elapsed < msPerMinute) {
		return Math.floor(elapsed/1000) + 's ago';
	}

	if (elapsed < msPerHour) {
		return Math.floor(elapsed/msPerMinute) + 'm ago';
	}

	return `${Math.floor(elapsed/msPerHour)}h, ${Math.floor(elapsed/msPerMinute) % 60}m ago`;
}

function create_popup(cb: (container: HTMLDivElement, open: () => any, close: () => any) => any) {
	var container = document.createElement('div');
	container.className = 'popup-container';

	function close() {
		container.parentElement!.removeChild(container);
	}

	container.addEventListener('click', e => {
		if ((<HTMLElement>e.target) == container) {
			close()
		}
	});

	var inner = document.createElement('div');
	inner.className = 'popup';
	container.appendChild(inner);

	cb(inner, () => document.body.appendChild(container), close);
}

function for_each<I, R>(items: I[], next_item: (item: I, item_finished: (...items: R[]) => any) => any, on_fin?: (resp: R[][]) => any) {
	var pos = 0;
	var finished: R[][] = [];

	next();

	function next() {
		if (items.length == pos) {
			return on_fin && on_fin(finished);
		}

		next_item(items[pos++], (...resp: R[]) => {
			finished.push(resp);
			next();
		});
	}
}