import View from '../index';
import EditorView from './editor';
import FilterView from './filter';
import FeedsView from './feeds';
import Dashboard from '../dashboard';

import { newEmptyPopup } from '../../util/popup';

import { notifyErrorDesc } from '../../util/notification';

import core, { for_each } from '../../core';

import { FeedListener, FeedItem } from '../../process';

import {
	send_get_item_list,
	send_create_category,
	send_get_category_list,
	send_remove_feed_from_category,
	send_add_feed_to_category
} from '../../socket';

export class Table {
	container = document.createElement('div');

	showing_item_content = <Nullable<HTMLDivElement>>null;

	viewing_category = <Nullable<number>>null;
	viewing_feed = <Nullable<number>>null;

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
		this.current_section = -1;
		this.row_ids = [];
		this.rows = [];
	}

	public init() {
		// Set rows. (Otherwise it won't have them if the table loads after the items are grabbed.)
		this.rows = core.process.feed_items.map(i => new TableItem(this, i));

		setTimeout(() => {
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
		}, 100);
	}

	public render(): HTMLDivElement {
		while (this.container.firstChild != null) {
			this.container.removeChild(this.container.firstChild);
		}

		this.render_rows(this.rows);

		return this.container;
	}

	public render_rows(rows: TableItem[]) {
		let section_names = [
			'Today',
			'Yesterday',
			'This Week',
			'This Month',
			'Last Month',
			'This Year',
			'Last Year'
		];

		rows.forEach(r => {
			let section = get_section_from_date(r.feed_item.date * 1000);
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
			if (timestamp < now - (day * 7)) return 2;

			// Yesterday
			if (timestamp < now - day) return 1;

			return 0;
		}
	}


	public new_items(resp: ItemListResponse, items: FeedItem[]) {
		this.last_skip_amount = resp.skip_count;
		this.last_req_amount = resp.item_count;
		this.last_total_items = resp.total_items;

		this.add_sort_render_rows(items);
	}

	public add_sort_render_rows(items: FeedItem[]) {
		let table_items = items
			.filter(i => this.row_ids.indexOf(i.id!) == -1)
			.map(i => {
				this.row_ids.push(i.id!);
				return new TableItem(this, i);
			});

		this.rows = this.rows.concat(table_items);

		this.rows.sort(this.sort_item_func('date', 1));
		this.row_ids.sort();

		this.render();

		return table_items;
	}

	public grab_new_rows(going_backwards: boolean) {
		if (this.waiting_for_more_feeds) return;

		this.waiting_for_more_feeds = true;

		// Scrolling Upwards
		if (going_backwards) {}

		let skip_amount = this.last_skip_amount + this.last_req_amount;

		// If next request is going to be outside of the the total items we have, return.
		if (skip_amount > this.last_total_items) {
			this.waiting_for_more_feeds = true;
			return;
		}

		console.log('Skipping:', skip_amount + '/' + this.last_total_items);

		send_get_item_list(this.viewing_category, skip_amount, 25)
		.then(resp => {
			let feed_items = resp.items.map(i => new FeedItem(i, resp.notification_ids.includes(i.id!)));
			feed_items = core.process.add_or_update_feed_items(feed_items);

			this.new_items(resp!, feed_items);

			this.waiting_for_more_feeds = false;
		})
		.catch(err => {
			notifyErrorDesc('Grabbing Feed Item List', err);
			this.waiting_for_more_feeds = false;
			console.error(err);
		})
	}

	public remove_items_by_id(item_id: number[] | number) {
		if (Array.isArray(item_id)) {
			item_id.forEach(i => this.remove_items_by_id(i));
		} else {
			let index = this.row_ids.indexOf(item_id);
			if (index != -1) {
				this.row_ids.splice(index, 1);

				for (let i = 0; i < this.rows.length; i++) {
					if (this.rows[i].feed_item.id == item_id) {
						this.rows.splice(i, 1);
						break;
					}
				}
			}
		}
	}

	public sort_item_func(sort_method: 'id' | 'date_added' | 'date', sort_order: 1 | -1): (a: TableItem, b: TableItem) => number {
		return (a, b) => (b.feed_item[sort_method] - a.feed_item[sort_method]) * sort_order;
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

	feed_item: FeedItem;
	table: Table;

	// Element
	date_element = document.createElement('span');

	constructor(table: Table, feed_item: FeedItem) {
		this.table = table;
		this.feed_item = feed_item;

		this.container.className = 'feed-item';

		if (feed_item.alert) {
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

		// feed site name
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item feed-name';

			let feed = core.process.get_feed_by_id(this.feed_item.feed_id)!;

			let span = document.createElement('a');
			span.innerText = feed.generator || feed.title;
			span.title = span.innerText;
			span.href = `/?feed=${this.feed_item.feed_id}`;
			li.appendChild(span);

			return li;
		})());

		// Title
		list.appendChild((() => {
			let li = document.createElement('li');
			li.className = 'list-item title';

			let span = document.createElement('a');
			span.className = 'default';
			span.innerText = this.feed_item.title;
			span.title = span.innerText;
			span.href = this.feed_item.link;

			span.addEventListener('click', e => {
				let showing_content = this.table.showing_item_content;

				if (showing_content != null) {
					this.container.removeChild(showing_content);
					this.table.showing_item_content = null;
				} else {
					showing_content = document.createElement('div');
					showing_content.style.padding = '10px';
					showing_content.innerHTML = this.feed_item.content;

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
			this.date_element.title = new Date(this.feed_item.date * 1000).toLocaleString();
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
			a_href.href = this.feed_item.link;
			li.appendChild(a_href);

			return li;
		})());

		return this.container;
	}


	public parse_timestamp(): string {
		return this.feed_item.parse_timestamp();
	}

	public update_date_element() {
		this.date_element.innerText = this.parse_timestamp();
	}

	static HEIGHT = 41;
}


export default class FeedView extends View {
	table = new Table();

	nav_bar = document.createElement('div');
	nav_bar_list = document.createElement('ul');

	categories = <SidebarItem[]>[];

	static path = 'feeds';

	constructor() {
		super(FeedView.path);
	}


	on_init() {
		// Navbar
		this.nav_bar.className = 'nav-bar';

		let title_container = document.createElement('div');
		title_container.className = 'title-container';
		this.nav_bar.appendChild(title_container);

		let title = document.createElement('h1');
		title.className = 'title';
		title.innerText = 'Watchmen';
		title_container.appendChild(title);

		let nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		this.nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(this.nav_bar);

		this.container.appendChild(this.table.render());

		// Get Categories (init)
		send_get_category_list()
		.then(opts => {
			console.log('Categories:', opts);
			this.create_nav_items(opts);
		})
		.catch(e => notifyErrorDesc('Grabbing Category List', e));
	}

	on_open() {
		console.log('open');

		const url_params = new URLSearchParams(location.search.slice(1));

		let cat_str = url_params.get('cat');
		this.table.viewing_category = cat_str ? parseInt(cat_str) : null;

		// TODO: ability to only see one feed.
		let feed_str = url_params.get('feed');
		this.table.viewing_feed = feed_str ? parseInt(feed_str) : null;

		if (this.table.viewing_category != null && this.table.viewing_feed != null) {
			window.location.pathname = '/?cat=' + this.table.viewing_category;
		}

		// Navbar buttons

		let dashboard = document.createElement('div');
		dashboard.className = 'button';
		dashboard.innerText = 'Dashboard';
		core.navbar.append_left_html(dashboard);

		dashboard.addEventListener('click', () => core.open_view(new Dashboard()));

		let new_listener = document.createElement('div');
		new_listener.className = 'button';
		new_listener.innerText = 'Watching';
		core.navbar.append_left_html(new_listener);

		new_listener.addEventListener('click', () => core.open_view(new FeedsView()));

		let open_editor = document.createElement('div');
		open_editor.className = 'button';
		open_editor.innerText = 'Editor';
		core.navbar.append_left_html(open_editor);

		open_editor.addEventListener('click', () => core.open_view(new EditorView()));


		let open_filter = document.createElement('div');
		open_filter.className = 'button';
		open_filter.innerText = 'Filters';
		core.navbar.append_left_html(open_filter);

		open_filter.addEventListener('click', () => core.open_view(new FilterView()));

		// Right side

		let search = document.createElement('input');
		search.placeholder = 'Filter Feed Items';
		search.type = 'text';
		core.navbar.append_right_html(search);

		let last_timeout_id: Nullable<number> = null;

		search.addEventListener('keydown', () => {
			if (last_timeout_id != null) {
				clearTimeout(last_timeout_id);
			}

			last_timeout_id = setTimeout(() => {
				last_timeout_id = null;

				if (search.value.length == 0) {
					//
				} else {
					//
				}
			}, 500);
		});

		// let show_listeners = document.createElement('div');
		// show_listeners.className = 'button';
		// show_listeners.innerText = 'Listeners';
		// nav_right.appendChild(show_listeners);

		// show_listeners.addEventListener('click', _ => {
		// 	create_popup((container, open, close) => {
		// 		//

		// 		open();
		// 	});
		// });
	}

	on_close() {
		core.navbar.clear();
	}


	create_nav_items(opts: CategoryListResponse) {
		// New Category Button
		let create_button = document.createElement('div');
		create_button.className = 'button new-category';
		create_button.innerText = 'New Category';
		this.nav_bar.appendChild(create_button);

		create_button.addEventListener('click', () => {
			const popup = newEmptyPopup();

			let form = document.createElement('div');
			form.className = 'form-group';
			popup.inner_container.appendChild(form);

			// Category Text
			let cat_row = document.createElement('div');
			cat_row.className = 'form-row';
			form.appendChild(cat_row);

			let cat_text = document.createElement('input');
			cat_text.placeholder = 'Category Name';
			cat_text.type = 'text';
			cat_row.appendChild(cat_text);

			// Submit
			let sub_row = document.createElement('div');
			sub_row.className = 'form-row';
			form.appendChild(sub_row);

			let submit = document.createElement('div');
			submit.className = 'button';
			submit.innerText = 'Create';
			sub_row.appendChild(submit);

			submit.addEventListener('click', _ => {
				let cat_id = this.largest_category_id();

				console.log(cat_text.value);
				console.log('Largest Category: ' + cat_id);

				send_create_category(cat_text.value, cat_id)
				.then(() => {
					send_get_category_list()
					.then(opts => this.create_categories(opts))
					.catch(e => notifyErrorDesc('Grabbing Category List', e));
				})
				.catch(e => notifyErrorDesc('Creating Category', e));

				popup.close();
			});

			popup.open();
		});

		this.create_categories(opts);
	}

	create_categories(opts: CategoryListResponse) {
		while (this.nav_bar_list.firstChild) this.nav_bar_list.firstChild.remove();

		let opts_items: ModelCategory[] = [
			{ id: -1, name: "All", name_lowercase: "all", date_added: 0, position: -1 }
		];
		opts_items = opts_items.concat(opts.categories);
		opts_items.sort((a, b) => a.position - b.position);

		let items = opts_items.map(i => new SidebarItem(this, i, opts.category_feeds.filter(f => f.category_id == i.id)));
		items.forEach(i => this.nav_bar_list.appendChild(i.render()));

		this.categories = items;
	}

	refresh_categories() {
		send_get_category_list()
		.then(opts => this.create_categories(opts))
		.catch(e => notifyErrorDesc('Grabbing Category List', e));
	}

	get_category_by_id(id: number): Nullable<SidebarItem> {
		for (let i = 0; i < this.categories.length; i++) {
			let feed = this.categories[i];

			if (feed.id == id) {
				return feed;
			}
		}

		return null;
	}

	largest_category_id(): number {
		let largest = 0;

		this.categories.forEach(c => c.id > largest ? largest = c.id : null);

		return largest;
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

	view: FeedView;

	constructor(view: FeedView, opts: ModelCategory, category_feeds?: ModelFeedCategory[]) {
		this.view = view;

		this.id = opts.id!;
		this.name = opts.name;
		this.name_lowercase = opts.name_lowercase;
		this.date_added = opts.date_added;
		this.position = opts.position;

		this.category_feeds = category_feeds || [];
	}

	public render(): HTMLLIElement {
		let container = document.createElement('li');
		container.className = 'tree-item' + (
			(this.view.table.viewing_category == null && this.id == -1)
			|| this.view.table.viewing_category == this.id ? ' active' : '');

		// Options
		if (this.id != -1) {
			let options_container = document.createElement('div');
			options_container.className = 'tree-item-options';
			options_container.innerText = 'O';
			container.appendChild(options_container);

			options_container.addEventListener('click', _ => this.show_editor());
		}

		// Title

		let title_container = document.createElement('a');
		title_container.className = 'tree-item-title';
		title_container.href = '/' + (this.id == -1 ? '' : `?cat=${this.id}`);
		container.appendChild(title_container);

		let title = document.createElement('span');
		title.className = 'title';
		title.innerText = this.name;
		title_container.appendChild(title);

		return container;
	}

	public show_editor() {
		const popup = newEmptyPopup();

		let form = document.createElement('div');
		form.className = 'form-group';
		popup.inner_container.appendChild(form);

		// Category Text
		let cat_row = document.createElement('div');
		cat_row.className = 'form-row';
		form.appendChild(cat_row);

		let inputs: [HTMLInputElement, number][] = [];

		// Display Feeds

		core.process.feed_listeners.forEach(feed => {
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
			let generator = document.createElement('div');
			generator.innerText = feed.generator.slice(0, 20) + (feed.generator.length > 20 ? '..' : '');
			generator.title = feed.generator;
			cont.appendChild(generator);

			// Description
			let descr = document.createElement('pre');
			descr.innerText = feed.description.slice(0, 80) + (feed.description.length > 80 ? '..' : '');
			descr.title = feed.description;
			cont.appendChild(descr);

			cat_row.appendChild(cont);

			inputs.push([input, feed.id]);
		});


		// TODO: Checkbox saying if browser should Notify

		// Submit
		let sub_row = document.createElement('div');
		sub_row.className = 'form-row';
		form.appendChild(sub_row);

		let submit = document.createElement('div');
		submit.className = 'button';
		submit.innerText = 'Update';
		sub_row.appendChild(submit);

		submit.addEventListener('click', _ => {
			let to_send_adding: number[] = [];
			let to_send_removing: number[] = [];

			inputs.forEach(input_id => {
				let input = input_id[0],
					feed_id = input_id[1];

				let contains_feed = this.contains_cat_feed_by_feed_id(feed_id);

				// Adding Feed
				if (!contains_feed && input.checked) {
					to_send_adding.push(feed_id);
				}

				// Removing Feed
				else if (contains_feed && !input.checked) {
					let feed = this.get_cat_feed_by_feed_id(feed_id);
					if (feed == null) return console.log('No category feed: ' + feed_id + ',', this.category_feeds);

					to_send_removing.push(feed.id!);
				}
			});

			let other_finished = false;

			if (to_send_adding.length != 0) {
				for_each(to_send_adding, (feed_id, fin) => {
					send_add_feed_to_category(feed_id, this.id)
					.then(fin)
					.catch(e => notifyErrorDesc('Adding Feed to Category', e));
				}, _ => {
					if (other_finished) {
						this.view.refresh_categories();

						if (this.view.table.viewing_category == this.id) {
							core.process.init_feeds();
						}
					}

					other_finished = true;
				});
			}

			if (to_send_removing.length != 0) {
				for_each(to_send_removing, (feed_id, fin) => {
					send_remove_feed_from_category(feed_id)
					.then(fin)
					.catch(e => notifyErrorDesc('Removing Feed from Category', e));
				}, _ => {
					if (other_finished) {
						this.view.refresh_categories();

						if (this.view.table.viewing_category == this.id) {
							core.process.init_feeds();
						}
					}

					other_finished = true;
				});
			}

			popup.close();
		});

		popup.open();
	}

	public get_cat_feed_by_id(id: number): Nullable<ModelFeedCategory> {
		if (this.category_feeds != null) {
			for (let i = 0; i < this.category_feeds.length; i++) {
				let element = this.category_feeds[i];
				if (element.id == id) return element;
			}
		}

		return null;
	}

	public get_cat_feed_by_feed_id(id: number): Nullable<ModelFeedCategory> {
		if (this.category_feeds != null) {
			for (let i = 0; i < this.category_feeds.length; i++) {
				let element = this.category_feeds[i];
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
			for (let i = 0; i < this.category_feeds.length; i++) {
				let element = this.category_feeds[i];
				if (element.id == id) {
					this.category_feeds.splice(i, 1);

					break;
				}
			}
		}
	}
}