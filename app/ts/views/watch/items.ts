import { elapsed_to_time_ago } from '../../process';
import { createElement } from '../../util/html';

import View from '../index';
import FeedItemsView from '../feed/items';
import DashboardView from '../dashboard';

import core, { create_popup } from '../../core';

import {
	send_get_watcher_list,
	send_create_watcher
} from '../../socket';

export default class WatchItemsView extends View {
	nav_bar = document.createElement('div');
	nav_bar_list = document.createElement('ul');

	table = new Table();

	constructor() {
		super();
	}


	on_init() {
		// Navbar
		this.nav_bar.className = 'nav-bar';

		let title_container = document.createElement('div');
		title_container.className = 'title-container';
		this.nav_bar.appendChild(title_container);

		let title = document.createElement('h1');
		title.className = 'title';
		title.innerText = 'Watching';
		title_container.appendChild(title);

		const create_button = createElement('div', { className: 'button new-category', innerText: 'New Feed'}, this.nav_bar);

		create_button.addEventListener('click', () => {
			create_popup((container, open, close) => {
				const form = createElement('div', { className: 'form-group' }, container);

				// Feed URL
				const cat_row = createElement('div', { className: 'form-row' }, form);
				const cat_text = createElement('input', { placeholder: 'Feed URL', type: 'text' }, cat_row);

				// Submit
				const sub_row = createElement('div', { className: 'form-row' }, form);
				const submit = createElement('div', { className: 'button', innerText: 'Create'}, sub_row);

				submit.addEventListener('click', _ => {
					send_create_watcher(cat_text.value, null, (err, opts) => {
						if (err != null || opts == null) {
							return console.error('create_watcher: ', err);
						}

						console.log('create_watcher:', opts);

						if (opts.affected != 0) {
							core.process.refresh_feeds(close);
						}
					});
				});

				open();
			});
		})

		// Nav bar items

		let nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		this.nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(this.nav_bar);

		this.container.appendChild(this.table.render());

		if (core.socket.is_open()) {
			this.on_connection_open();
		} else {
			core.socket.socket.addEventListener('open', _ => this.on_connection_open());
		}
	}

	on_connection_open() {
		//
	}

	on_open() {
		console.log('open');

		const url_params = new URLSearchParams(location.search.slice(1));

		// Navbar buttons

		let dashboard_listener = document.createElement('div');
		dashboard_listener.className = 'button';
		dashboard_listener.innerText = 'Dashboard';
		core.navbar.append_left_html(dashboard_listener);

		dashboard_listener.addEventListener('click', () => core.open_view(new DashboardView()));

		let feed_listener = document.createElement('div');
		feed_listener.className = 'button';
		feed_listener.innerText = 'Feeds';
		core.navbar.append_left_html(feed_listener);

		feed_listener.addEventListener('click', () => core.open_view(new FeedItemsView()));
	}

	on_close() {
		core.navbar.clear();
	}
}

export class Table {
	container = document.createElement('div');

	showing_item_content = <Nullable<HTMLDivElement>>null;

	viewing_watcher = <Nullable<number>>null;

	row_ids: number[] = [];
	rows: TableItem[] = [];

	// class for infinite-scroll or page buttons

	// filters = [];

	// Set from get_items
	last_req_amount = 0;
	last_skip_amount = 0;
	last_total_items = 0;

	waiting_for_more_feeds = false;

	constructor() {
		this.container.className = 'feed-items-container';
		this.init();
	}

	public reset() {
		while (this.container.firstChild != null) {
			this.container.removeChild(this.container.firstChild);
		}

		this.showing_item_content = null;
		this.last_req_amount = 0;
		this.last_skip_amount = 0;
		this.last_total_items = 0;
		this.waiting_for_more_feeds = false;
		this.row_ids = [];
		this.rows = [];
	}

	public init() {
		// Set rows. (Otherwise it won't have them if the table loads after the items are grabbed.)
		this.rows = core.process.watching_listeners.map(i => new TableItem(this, i[0], i[1]));
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild != null) {
			this.container.firstChild.remove();
		}

		this.rows.forEach(r => this.container.appendChild(r.render()));

		return this.container;
	}

	public add_sort_render_rows() {
		this.rows = core.process.watching_listeners.map(i => new TableItem(this, i[0], i[1]));

		this.rows.sort(this.sort_item_func('date_added', 1));
		this.row_ids.sort();

		this.render();
	}

	public sort_item_func(sort_method: 'date_added', sort_order: 1 | -1): (a: TableItem, b: TableItem) => number {
		return (a, b) => (b.history[sort_method] - a.history[sort_method]) * sort_order;
	}

	public can_continue_scroll_up(): boolean {
		return this.last_skip_amount != 0;
	}

	public can_continue_scroll_down(): boolean {
		return this.last_skip_amount  + this.last_req_amount < this.last_total_items;
	}

	public get_newest_timestamp(): number {
		return core.process.get_newest_timestamp();
	}
}

export class TableItem {
	container = document.createElement('div');

	watcher: ModelWatcher;
	history: ModelWatchHistory;

	table: Table;

	// Element
	date_element = document.createElement('span');

	constructor(table: Table, watcher: ModelWatcher, history: ModelWatchHistory) {
		this.table = table;
		this.watcher = watcher;
		this.history = history;

		this.container.className = 'feed-item';

		if (watcher.alert) {
			this.container.classList.add('notification');
		}
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild) {
			this.container.removeChild(this.container.firstChild);
		}

		let list = document.createElement('ul');
		list.className = 'list horizontal';
		this.container.appendChild(list);


		// Watcher Name
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item feed-name';

			let span = document.createElement('a');
			span.innerText = this.watcher.title;
			span.title = span.innerText;
			span.href = `/?watcher=${this.watcher.id}`;
			li.appendChild(span);

			return li;
		})());

		// Title
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item title';

			let span = document.createElement('a');
			span.className = 'default';
			span.innerText = this.history.value;
			span.title = span.innerText;

			span.addEventListener('click', e => {
				let showing_content = this.table.showing_item_content;

				if (showing_content != null) {
					this.container.removeChild(showing_content);
					this.table.showing_item_content = null;
				} else {
					showing_content = document.createElement('div');
					showing_content.style.padding = '10px';
					showing_content.innerText = 'TODO';

					this.container.appendChild(showing_content);

					this.table.showing_item_content = showing_content;
				}

				e.preventDefault();
				return false;
			});

			li.appendChild(span);

			return li;
		})());

		// Date
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item date';

			this.update_date_element();
			this.date_element.title = new Date(this.history.date_added * 1000).toLocaleString();
			li.appendChild(this.date_element);

			return li;
		})());

		// site link
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item link';

			let a_href = document.createElement('a');
			a_href.className = 'default';
			a_href.innerText = 'link';
			a_href.target = '_blank';
			a_href.href = this.watcher.url;
			li.appendChild(a_href);

			return li;
		})());

		return this.container;
	}


	public parse_timestamp(): string {
		let date = this.history.date_added * 1000;

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