import View from './index';
import ItemsView from './items';

import core, { create_popup, for_each } from '../core';

import { FeedListener } from '../process';

import {
	send_new_feed_filter,
	send_remove_feed_filter,
	send_get_filter_list,
	send_get_feed_list,
	send_get_custom_items_list,
	send_create_listener,
	send_remove_listener,
	send_edit_listener
} from '../socket';


export default class FeedsView extends View {
	table = new FeedTable();

	constructor() {
		super();
	}

	on_init() {
		this.render_sidebar();
		this.render_editor();
	}

	on_open() {
		// Navbar buttons
		let open_editor = document.createElement('div');
		open_editor.className = 'button';
		open_editor.innerText = 'Item Viewer';
		core.navbar.append_left_html(open_editor);

		open_editor.addEventListener('click', () => {
			if (this.parent != null) {
				core.open_view(this.parent);
			} else {
				core.open_view(new ItemsView());
			}
		});
	}

	on_close() {
		core.navbar.clear();
	}

	render_sidebar() {
		const nav_bar = document.createElement('div');
		this.container.appendChild(nav_bar);
		nav_bar.className = 'nav-bar';

		const title_container = document.createElement('div');
		title_container.className = 'title-container';
		nav_bar.appendChild(title_container);

		const title = document.createElement('h1');
		title.className = 'title';
		title.innerText = 'Feeder';
		title_container.appendChild(title);

		let create_button = document.createElement('div');
		create_button.className = 'button new-category';
		create_button.innerText = 'New Feed';
		nav_bar.appendChild(create_button);

		create_button.addEventListener('click', () => {
			create_popup((container, open, close) => {
				let form = document.createElement('div');
				form.className = 'form-group';
				container.appendChild(form);


				// Feed URL
				let cat_row = document.createElement('div');
				cat_row.className = 'form-row';
				form.appendChild(cat_row);

				let cat_text = document.createElement('input');
				cat_text.placeholder = 'Feed URL';
				cat_text.type = 'text';
				cat_row.appendChild(cat_text);


				// Custom Items
				let custom_row = document.createElement('div');
				custom_row.className = 'form-row';
				form.appendChild(custom_row);

				let custom_item_sel = document.createElement('select');
				custom_item_sel.name = 'custom_item';
				custom_row.appendChild(custom_item_sel);

				let cidefault = document.createElement('option');
				cidefault.innerText = 'Pick a Custom Item';
				cidefault.value = '';
				cidefault.disabled = true;
				cidefault.selected = true;
				custom_item_sel.appendChild(cidefault);

				send_get_custom_items_list((_, resp) => {
					if (resp != null) {
						resp.items.forEach(item => {
							let option = document.createElement('option');
							console.log(item);
							option.innerText = item.title;
							option.title = item.description;
							option.value = '' + item.id!;

							custom_item_sel.appendChild(option);
						});
					}
				});

				// Submit
				let sub_row = document.createElement('div');
				sub_row.className = 'form-row';
				form.appendChild(sub_row);

				let submit = document.createElement('div');
				submit.className = 'button';
				submit.innerText = 'Create';
				sub_row.appendChild(submit);

				submit.addEventListener('click', _ => {
					if (custom_item_sel.value.length == 0) return;

					send_create_listener(cat_text.value, parseInt(custom_item_sel.value), (err, opts) => {
						if (err != null) {
							return console.error('create_listener: ', err);
						}

						console.log('create_listener:', opts);

						if (opts!.affected != 0) {
							core.process.refresh_feeds(close);
						}
					});
				});

				open();
			});
		})
	}

	render_editor() {
		const container = document.createElement('div');
		container.className = 'feeds-container';
		this.container.appendChild(container);

		this.table.render(container);
	}
}


class FeedTable {
	container = document.createElement('div');

	constructor() {
		this.container.className = 'feed-table';
	}

	render(parent: HTMLElement) {
		parent.appendChild(this.container);
		this.update();
	}

	update() {
		send_get_feed_list((err, resp) => {
			if (err != null) return console.error(err);

			let items = resp!.items;

			items.forEach(i => this.container.appendChild(new FeedItem(i).render()));
		});
	}
}


class FeedItem {
	model: ModelListener;
	editor?: HTMLDivElement;

	constructor(model: ModelListener) {
		this.model = model;
	}

	render() {
		let container = document.createElement('div');
		container.className = 'table-item';


		const display = document.createElement('div');
		display.className = 'display';
		container.appendChild(display);


		// Info

		let info = document.createElement('div');
		info.className = 'info';
		display.appendChild(info);

		info.appendChild(this.render_cell(this.model.title, 'title'));
		info.appendChild(this.render_cell(this.model.description, 'small'));
		info.appendChild(this.render_cell('URL: ' + this.model.url, 'small'));
		info.appendChild(this.render_cell(`Interval: ${this.model.sec_interval / 60} minutes`, 'small'));
		info.appendChild(this.render_cell(`Showing ${this.model.global_show ? 'Globally' : 'in Categories'}`, 'small'));
		info.appendChild(this.render_cell(`${this.model.remove_after == 0 ? 'Never removing old items' : 'Removing feeds after ' + this.model.remove_after + ' seconds'}, ${this.model.ignore_if_not_new ? 'Fetching only NEW items' : 'Fetching ALL items'}`, 'small'));


		// Buttons

		let buttons = document.createElement('div');
		buttons.className = 'buttons';
		display.appendChild(buttons);

		let edit_button = document.createElement('div');
		edit_button.className = 'button warning';
		edit_button.innerText = 'Edit';
		buttons.appendChild(edit_button);

		let delete_button = document.createElement('div');
		delete_button.className = 'button danger';
		delete_button.innerText = 'Remove';
		buttons.appendChild(delete_button);

		edit_button.addEventListener('click', () => {
			if (this.editor) {
				this.editor.remove();
				this.editor = undefined;
			} else {
				container.appendChild(this.render_editor());
			}
		});

		delete_button.addEventListener('click', () => {
			// Options on what to remove
			create_popup((container, open, close) => {
				let sub_row = document.createElement('div');
				sub_row.className = 'form-row';
				container.appendChild(sub_row);

				// Partial Delete
				let partial = document.createElement('div');
				partial.className = 'button';
				partial.innerText = 'Partial Delete (Only remove listener)';
				sub_row.appendChild(partial);

				// Fully Delete
				let fully = document.createElement('div');
				fully.className = 'button';
				fully.innerText = 'Fully Delete (Remove listener AND feed items)';
				sub_row.appendChild(fully);

				const removeChoice = (full: boolean) => {
					fully.removeEventListener('click', full_func);
					partial.removeEventListener('click', partial_func);

					send_remove_listener(this.model.id!, full, (err, opts) => {
						console.log('send_remove_listener:', err, opts);

						core.process.refresh_feeds(close);
					});
				};

				let self = this;
				const partial_func = () => removeChoice.call(self, false);
				const full_func = () => removeChoice.call(self, true);

				partial.addEventListener('click', partial_func);
				fully.addEventListener('click', full_func);

				open();
			});

		});

		return container;
	}

	render_cell(title: string, clazz: string) {
		let cell = document.createElement('div');
		cell.className = 'item-row ' + clazz;
		cell.innerText = title;
		return cell;
	}

	render_editor() {
		const container = document.createElement('div');
		container.className = 'editor';
		this.editor = container;


		// Editor Title
		const editor_title = document.createElement('div');
		editor_title.className = 'title';
		editor_title.innerText = 'Editor';
		container.appendChild(editor_title);


		// Contents
		const contents = document.createElement('div');
		contents.className = 'contents';
		container.appendChild(contents);


		// Editor Form
		const form = document.createElement('div');
		form.className = 'form-group';
		contents.appendChild(form);


		// Title
		const title_container = document.createElement('div');
		title_container.className = 'form-row';
		form.appendChild(title_container);

		title_container.appendChild(title('Title'));

		const title_input = document.createElement('input');
		title_input.value = this.model.title;
		title_input.addEventListener('change', () => this.model.title = title_input.value);
		title_input.type = 'text';
		title_container.appendChild(title_input);


		// Description
		const desc_container = document.createElement('div');
		desc_container.className = 'form-row';
		form.appendChild(desc_container);

		desc_container.appendChild(title('Description'));

		const desc_input = document.createElement('textarea');
		desc_input.value = this.model.description;
		desc_input.addEventListener('change', () => this.model.description = desc_input.value);
		desc_container.appendChild(desc_input);


		// URL
		const url_container = document.createElement('div');
		url_container.className = 'form-row';
		form.appendChild(url_container);

		url_container.appendChild(title('URL'));

		const url_input = document.createElement('input');
		url_input.value = this.model.url;
		url_input.addEventListener('change', () => this.model.url = url_input.value);
		url_input.type = 'text';
		url_container.appendChild(url_input);


		// Interval
		const interval_container = document.createElement('div');
		interval_container.className = 'form-row';
		form.appendChild(interval_container);

		interval_container.appendChild(title('Interval'));

		const interval_input = document.createElement('input');
		interval_input.value = '' + this.model.sec_interval;
		interval_input.addEventListener('change', () => this.model.sec_interval = interval_input.valueAsNumber);
		interval_input.type = 'number';
		interval_container.appendChild(interval_input);


		// Remove After X seconds.
		const auto_remove_container = document.createElement('div');
		auto_remove_container.className = 'form-row';
		form.appendChild(auto_remove_container);

		auto_remove_container.appendChild(title('Auto Remove After X time?'));

		const auto_rem_input = document.createElement('input');
		auto_rem_input.value = '' + this.model.remove_after;
		auto_rem_input.addEventListener('change', () => this.model.remove_after = auto_rem_input.valueAsNumber);
		auto_rem_input.type = 'number';
		auto_remove_container.appendChild(auto_rem_input);


		// Fetching new or any
		const fetch_type_container = document.createElement('div');
		fetch_type_container.className = 'form-row';
		form.appendChild(fetch_type_container);

		fetch_type_container.appendChild(title('Fetch New Only?'));

		const fetch_type_input = document.createElement('input');
		fetch_type_input.checked = this.model.ignore_if_not_new;
		fetch_type_input.addEventListener('change', () => this.model.ignore_if_not_new = fetch_type_input.checked);
		fetch_type_input.type = 'checkbox';
		fetch_type_container.appendChild(fetch_type_input);


		function title(name: string) {
			const text = document.createElement('label');
			text.innerText = name;
			return text;
		}


		// Submit
		const buttons_container = document.createElement('div');
		buttons_container.className = 'form-row';
		form.appendChild(buttons_container);

		const submit_button = document.createElement('div');
		submit_button.className = 'button success';
		submit_button.innerText = 'Submit';
		buttons_container.appendChild(submit_button);


		// Filter Linking
		const filter_container = document.createElement('div');
		contents.appendChild(filter_container);

		// send_get_filter_list
		const linking_selection = document.createElement('select');
		linking_selection.className = 'custom';
		linking_selection.multiple = true;
		filter_container.appendChild(linking_selection);

		let default_filters: { [key: number]: boolean } = {};

		send_get_filter_list((err, filters) => {
			if (err != null) throw err;

			filters!.items.forEach(filter => {
				const option = document.createElement('option');

				option.selected = filter.feeds.indexOf(this.model.id!) != -1;
				option.innerText = '' + filter.filter.title;
				option.value = '' + filter.filter.id;

				default_filters[filter.filter.id] = option.selected;

				linking_selection.appendChild(option);
			});
		});

		submit_button.addEventListener('click', () => {
			// submit_button.innerText = 'Submitting. Please wait..';

			// Update Filters for Feed
			for (let i = 0; i < linking_selection.options.length; i++) {
				const option = linking_selection.options[i];
				let id = parseInt(option.value);
				console.log(id + ' - ' + option.selected);

				if (default_filters[id] != option.selected) {
					if (option.selected) {
						// Selected. Wasn't before. Enable it.
						send_new_feed_filter(this.model.id!, id, err => { if (err) { throw err; } });
					} else {
						// Not selected. Was before. Remove it.
						send_remove_feed_filter(this.model.id!, id, err => { if (err) { throw err; } });
					}
				}
			}

			// Update the Listener
			// send_edit_listener(this.model.id!, this.model, () => {
			// 	submit_button.innerText = 'Submit';
			// });
		});

		return container;
	}
}