import { elapsed_to_time_ago } from '../../process';
import { createElement } from '../../util/html';

import View from '../index';
import FeedItemsView from '../feed/items';
import DashboardView from '../dashboard';
import EditorView from './editor';

import core, { create_popup } from '../../core';

import {
	send_create_watcher,
	send_get_watch_parser_list,
	send_get_watch_history_list
} from '../../socket';

export default class WatchItemsView extends View {
	nav_bar = document.createElement('div');
	nav_bar_list = document.createElement('ul');

	table = new Table();

	static path = 'watcher';

	constructor() {
		super(WatchItemsView.path);
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

		const create_button = createElement('div', { className: 'button new-category', innerText: 'New Watcher'}, this.nav_bar);

		create_button.addEventListener('click', () => {
			create_popup((container, open, close) => {
				const form = createElement('div', { className: 'form-group' }, container);

				// Feed URL
				const cat_row = createElement('div', { className: 'form-row' }, form);
				const cat_text = createElement('input', { placeholder: 'Feed URL', type: 'text' }, cat_row);

				// Submit
				const sub_row = createElement('div', { className: 'form-row' }, form);
				const submit = createElement('div', { className: 'button', innerText: 'Create'}, sub_row);

				// Parser ID
				const parser_row = createElement('div', { className: 'form-row' }, form);
				const parser_item_sel = createElement('select', { name: 'custom_item' }, parser_row);

				createElement('option', { innerText: 'Pick a Watch Parser', value: '', disabled: true, selected: true }, parser_item_sel);

				send_get_watch_parser_list((_, resp) => {
					if (resp != null) {
						resp.items.forEach(item => {
							createElement('option', { innerText: item.title, title: item.description, value: '' + item.id }, parser_item_sel);
						});
					}
				})

				submit.addEventListener('click', _ => {
					if (parser_item_sel.value.length == 0) return;

					send_create_watcher(cat_text.value, parseInt(parser_item_sel.value), (err, opts) => {
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
		});

		// Nav bar items

		let nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		this.nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(this.nav_bar);

		this.container.appendChild(this.table.render());
	}

	on_connection_open() {
		// TODO: Change on_connection_open() to activate after base processes have been processed.
		setTimeout(() => this.table.regrab(), 1500);
	}

	on_open() {
		console.log('open');

		// Navbar buttons

		let dashboard_listener = document.createElement('div');
		dashboard_listener.className = 'button';
		dashboard_listener.innerText = 'Dashboard';
		core.navbar.append_left_html(dashboard_listener);

		dashboard_listener.addEventListener('click', () => core.open_view(new DashboardView()));

		let editor_listener = document.createElement('div');
		editor_listener.className = 'button';
		editor_listener.innerText = 'Editor';
		core.navbar.append_left_html(editor_listener);

		editor_listener.addEventListener('click', () => core.open_view(new EditorView()));
	}

	on_close() {
		core.navbar.clear();
	}
}

export class Table {
	container = document.createElement('div');

	showing_item_content = <Nullable<HTMLDivElement>>null;

	viewing_watcher = <Nullable<number>>null;

	rows: TableItem[] = [];

	current_section = -1;

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
		this.current_section = -1;
		this.waiting_for_more_feeds = false;
		this.rows = [];
	}

	public init() {
		// Set rows. (Otherwise it won't have them if the table loads after the items are grabbed.)
		this.regrab();
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild != null) {
			this.container.firstChild.remove();
		}

		let section_names = [
			'Today',
			'Yesterday',
			'This Week',
			'This Month',
			'Last Month',
			'This Year',
			'Last Year'
		];

		this.rows.forEach(r => {
			let section = get_section_from_date(r.history.date_added * 1000);

			if (section != this.current_section) {
				this.current_section = section;

				let section_name = section_names[section];

				let section_html = document.createElement('div');
				section_html.className = 'section ' + section_name.toLowerCase().replace(' ', '-');
				section_html.innerHTML = `<span>${section_name}</span>`;

				this.container.appendChild(section_html);
			}

			this.container.appendChild(r.render());
		});

		function get_section_from_date(timestamp: number): number {
			const now = Date.now();
			const day = 1000 * 60 * 60 * 24;

			// Last Year
			if (timestamp < now - (day * 365 * 2)) return 6;

			// This Year
			if (timestamp < now - (day * 365 * 2)) return 5;

			// Last Month
			if (timestamp < now - (day * 30 * 2)) return 4;

			// This Month
			if (timestamp < now - (day * 30)) return 3;

			// This Week
			// if (timestamp < now - (day * 7)) return 2;

			// Yesterday
			// if (timestamp < now - day) return 1;

			return 2;
		}

		return this.container;
	}

	public regrab() {
		// Set rows. (Otherwise it won't have them if the table loads after the items are grabbed.)
		this.rows = core.process.watching_listeners.map(i => new TableItem(this, i[0], i[1]));
		this.rows.sort((a, b) => b.history.date_added - a.history.date_added);
	}

	public add_sort_render_rows() {
		this.regrab();
		this.render();
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

		console.log(this);

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
			span.innerText = this.history.items[0].value;
			span.title = span.innerText;

			span.addEventListener('click', e => {
				let showing_content = this.table.showing_item_content;

				if (showing_content != null) {
					if (showing_content.parentElement != this.container) {
						showing_content.remove();
						this.table.showing_item_content = null;
					} else {
						showing_content.remove();
						this.table.showing_item_content = null;
						return;
					}
				}

				showing_content = document.createElement('div');
				showing_content.style.padding = '10px';
				showing_content.innerText = 'Loading...';

				send_get_watch_history_list(this.watcher.id!, 0, undefined, (_, resp) => {
					// No longer showing dropdown OR clicked off item quick enough? Return.
					if (showing_content == null || showing_content.parentElement != this.container) return;

					let list = resp!.items.map(i => `[${new Date(i.date_added * 1000).toLocaleString()}]: ${i.items[0].value}`);

					if (list.length == 0) {
						showing_content!.innerText = 'None.';
					} else {
						showing_content!.innerText = list.join('\n');
					}
				});

				this.container.appendChild(showing_content);

				this.table.showing_item_content = showing_content;

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