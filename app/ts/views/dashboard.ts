// Will be used to look at neweset feeds/watches at a glance.

import View from './index';
import FeedItemsView from './feed/items';
import WatchItemsView from './watch/items';

import core from '../core';

export default class DasboardView extends View {
	nav_bar = document.createElement('div');
	nav_bar_list = document.createElement('ul');

	static path = '';

	constructor() {
		super(DasboardView.path);
	}


	on_init() {
		// Navbar
		this.nav_bar.className = 'nav-bar';

		let title_container = document.createElement('div');
		title_container.className = 'title-container';
		this.nav_bar.appendChild(title_container);

		let title = document.createElement('h1');
		title.className = 'title';
		title.innerText = 'Dashboard';
		title_container.appendChild(title);

		let nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		this.nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(this.nav_bar);
	}


	on_open() {
		console.log('open');

		const url_params = new URLSearchParams(location.search.slice(1));

		// Navbar buttons

		let feed_listener = document.createElement('div');
		feed_listener.className = 'button';
		feed_listener.innerText = 'Feeds';
		core.navbar.append_left_html(feed_listener);

		feed_listener.addEventListener('click', () => core.open_view(new FeedItemsView()));


		let watch_listener = document.createElement('div');
		watch_listener.className = 'button';
		watch_listener.innerText = 'Watching';
		core.navbar.append_left_html(watch_listener);

		watch_listener.addEventListener('click', () => core.open_view(new WatchItemsView()));
	}

	on_close() {
		core.navbar.clear();
	}
}