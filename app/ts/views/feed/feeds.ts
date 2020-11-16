import View from '../index';
import ItemsView from './items';

import { createElement } from '../../util/html';
import { newEmptyPopup } from '../../util/popup';
import { notifyErrorDesc } from '../../util/notification';

import core from '../../core';

import { FeedListener } from '../../process';

import {
	send_new_feed_filter,
	send_remove_feed_filter,
	send_get_filter_list,
	send_get_feed_list,
	send_get_custom_items_list,
	send_create_listener,
	send_remove_listener,
	send_edit_listener
} from '../../socket';


export default class FeedsView extends View {
	table = new FeedTable();

	static path = 'feeds-list';

	constructor() {
		super(FeedsView.path);
	}

	on_init() {
		this.render_sidebar();
		this.render_editor();
	}

	on_open() {
		// Navbar buttons
		let open_editor = createElement(
			'div',
			{
				className: 'button',
				innerText: 'Item Viewer'
			}
		);

		core.navbar.append_left_html(open_editor);

		open_editor.addEventListener('click', () => core.open_view(new ItemsView()));
	}

	on_close() {
		core.navbar.clear();
	}

	render_sidebar() {
		const nav_bar = createElement('div', { className: 'nav-bar' }, this.container);

		const title_container = createElement('div', { className: 'title-container' }, nav_bar);
		createElement('h1', { className: 'title', innerText: 'Watchmen' }, title_container);

		const create_button = createElement('div', { className: 'button new-category', innerText: 'New Feed'}, nav_bar);

		create_button.addEventListener('click', () => {
			const popup = newEmptyPopup();

			const form = createElement('div', { className: 'form-group' }, popup.inner_container);

			// Feed URL
			const cat_row = createElement('div', { className: 'form-row' }, form);

			const cat_text = createElement('input', { placeholder: 'Feed URL', type: 'text' }, cat_row);

			// Custom Items
			const custom_row = createElement('div', { className: 'form-row' }, form);

			const custom_item_sel = createElement('select', { name: 'custom_item' }, custom_row);

			createElement('option', { innerText: 'Pick a Custom Item', value: '', disabled: true, selected: true }, custom_item_sel);

			send_get_custom_items_list()
			.then(resp => {
				if (resp != null) {
					resp.items.forEach(item => {
						createElement('option', { innerText: item.title, title: item.description, value: '' + item.id }, custom_item_sel);
					});
				}
			})
			.catch(e => notifyErrorDesc('Grabbing Custom Items List', e));

			// Submit
			const sub_row = createElement('div', { className: 'form-row' }, form);

			const submit = createElement('div', { className: 'button', innerText: 'Create'}, sub_row);

			submit.addEventListener('click', _ => {
				if (custom_item_sel.value.length == 0) return;

				send_create_listener(cat_text.value, parseInt(custom_item_sel.value))
				.then(opts => {
					console.log('create_listener:', opts);

					if (opts.affected != 0) {
						core.process.init_feeds()
						.then(close)
						.catch(e => notifyErrorDesc('Re-initiating Feeds', e));
					}
				})
				.catch(e => notifyErrorDesc('Creating Listener', e));
			});

			popup.open();
		});
	}

	render_editor() {
		const container = createElement('div', { className: 'feeds-container' }, this.container);

		this.table.render(container);
	}
}


class FeedTable {
	container = createElement('div', { className: 'feed-table' });

	render(parent: HTMLElement) {
		parent.appendChild(this.container);
		this.update();
	}

	update() {
		send_get_feed_list()
		.then(resp => {
			resp.items.forEach(i => this.container.appendChild(new FeedItem(i).render()));
		})
		.catch(e => notifyErrorDesc('Grabbing Feed List', e));
	}
}


class FeedItem {
	model: ModelListener;
	editor?: HTMLDivElement;

	constructor(model: ModelListener) {
		this.model = model;
	}

	render() {
		const container = createElement('div', { className: 'table-item' });

		const display = createElement('div', { className: 'display' }, container);

		// Info

		const info = createElement('div', { className: 'info' }, display);

		info.appendChild(this.render_cell(this.model.title, 'title'));
		info.appendChild(this.render_cell(this.model.description, 'small'));
		info.appendChild(this.render_cell('URL: ' + this.model.url, 'small'));
		info.appendChild(this.render_cell(`Interval: ${this.model.sec_interval / 60} minutes`, 'small'));
		info.appendChild(this.render_cell(`Showing ${this.model.global_show ? 'Globally' : 'in Categories'}`, 'small'));
		info.appendChild(this.render_cell(`${this.model.remove_after == 0 ? 'Never removing old items' : 'Removing feeds after ' + this.model.remove_after + ' seconds'}, ${this.model.ignore_if_not_new ? 'Fetching only NEW items' : 'Fetching ALL items'}`, 'small'));


		// Buttons

		const buttons = createElement('div', { className: 'buttons' }, display);

		const edit_button = createElement('div', { className: 'button warning', innerText: 'Edit' }, buttons);
		const delete_button = createElement('div', { className: 'button danger', innerText: 'Remove' }, buttons);

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
			const popup = newEmptyPopup();

			const sub_row = createElement('div', { className: 'form-row' }, popup.inner_container);

			const partial = createElement('div', { className: 'button', innerText: 'Partial Delete (Only remove listener)' }, sub_row);
			const fully = createElement('div', { className: 'button', innerText: 'Fully Delete (Remove listener AND feed items)' }, sub_row);

			const removeChoice = (full: boolean) => {
				fully.removeEventListener('click', full_func);
				partial.removeEventListener('click', partial_func);

				send_remove_listener(this.model.id!, full)
				.then(() => core.process.init_feeds().then(close, e => notifyErrorDesc('Re-initating Feeds', e)))
				.catch(e => notifyErrorDesc('Removing Listener', e));
			};

			let self = this;
			const partial_func = () => removeChoice.call(self, false);
			const full_func = () => removeChoice.call(self, true);

			partial.addEventListener('click', partial_func);
			fully.addEventListener('click', full_func);

			popup.open();
		});

		return container;
	}

	render_cell(title: string, clazz: string) {
		return createElement('div', { className: 'item-row ' + clazz, innerText: title });
	}

	render_editor() {
		const container = createElement('div', { className: 'editor' }, this.editor);

		// Editor Title
		createElement('div', { className: 'title', innerText: 'Editor' }, container);


		// Contents
		const contents = createElement('div', { className: 'contents' }, container);

		// Editor Form
		const form = createElement('div', { className: 'form-group' }, contents);


		// Title
		const title_container = createElement('div', { className: 'form-row' }, form);
		title_container.appendChild(label_title('Title'));

		const title_input = createElement('input', { value: this.model.title, type: 'text' }, title_container);
		title_input.addEventListener('change', () => this.model.title = title_input.value);


		// Description
		const desc_container = createElement('div', { className: 'form-row' }, form);
		desc_container.appendChild(label_title('Description'));

		const desc_input = createElement('textarea', { value: this.model.description }, desc_container);
		desc_input.addEventListener('change', () => this.model.description = desc_input.value);


		// URL
		const url_container = createElement('div', { className: 'form-row' }, form);
		url_container.appendChild(label_title('URL'));

		const url_input = createElement('input', { value: this.model.url, type: 'text' }, url_container);
		url_input.addEventListener('change', () => this.model.url = url_input.value);


		// Interval
		const interval_container = createElement('div', { className: 'form-row' }, form);
		interval_container.appendChild(label_title('Interval'));

		const interval_input = createElement('input', { value: '' + this.model.sec_interval, type: 'number' }, interval_container);
		interval_input.addEventListener('change', () => this.model.sec_interval = interval_input.valueAsNumber);


		// Remove After X seconds.
		const auto_remove_container = createElement('div', { className: 'form-row' }, form);
		auto_remove_container.appendChild(label_title('Auto Remove After X time?'));

		const auto_rem_input = createElement('input', { value: '' + this.model.remove_after, type: 'number' }, auto_remove_container);
		auto_rem_input.addEventListener('change', () => this.model.remove_after = auto_rem_input.valueAsNumber)


		// Fetching new or any
		const fetch_type_container = createElement('div', { className: 'form-row' }, form);
		fetch_type_container.appendChild(label_title('Fetch New Only?'));

		const fetch_type_input = createElement('input', { checked: this.model.ignore_if_not_new, type: 'checkbox' }, fetch_type_container);
		fetch_type_input.addEventListener('change', () => this.model.ignore_if_not_new = fetch_type_input.checked);


		function label_title(name: string) {
			return createElement('label', { innerText: name });
		}


		// Submit
		const buttons_container = createElement('div', { className: 'form-row' }, form);

		const submit_button = createElement('div', { className: 'button success', innerText: 'Submit' }, buttons_container);

		// Filter Linking
		const filter_container = createElement('div', undefined, contents);

		// send_get_filter_list
		const linking_selection = createElement('select', { className: 'custom', multiple: true }, filter_container);

		let default_filters: { [key: number]: boolean } = {};

		send_get_filter_list()
		.then(filters => {
			filters.items.forEach(filter => {
				const option = createElement('option', {
					selected: filter.feeds.indexOf(this.model.id!) != -1,
					innerText: '' + filter.filter.title,
					value: '' + filter.filter.id
				}, linking_selection);

				default_filters[filter.filter.id] = option.selected;
			});
		})
		.catch(e => notifyErrorDesc('Grabbing Filter List', e));

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
						send_new_feed_filter(this.model.id!, id)
						.catch(e => notifyErrorDesc('Creating Feed Filter', e));
						console.log('Added: ' + this.model.id! + ' - ' + id);
					} else {
						// Not selected. Was before. Remove it.
						send_remove_feed_filter(this.model.id!, id)
						.catch(e => notifyErrorDesc('Removing Feed Filter', e));
						console.log('Removed: ' + this.model.id! + ' - ' + id);
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