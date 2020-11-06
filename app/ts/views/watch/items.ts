import View from '../index';
import FeedItemsView from '../feed/items';
import DashboardView from '../dashboard';

import core from '../../core';

import {
	send_get_watcher_list,
	send_create_watcher
} from '../../socket';

export default class WatchItemsView extends View {
	nav_bar = document.createElement('div');
	nav_bar_list = document.createElement('ul');

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

		let nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		this.nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(this.nav_bar);

		if (core.socket.is_open()) {
			this.on_connection_open();
		} else {
			core.socket.socket.addEventListener('open', _ => this.on_connection_open());
		}
	}

	on_connection_open() {
		send_get_watcher_list((err, items) => {
			console.log(err);
			console.log(items);
		});
	}

	on_open() {
		console.log('open');

		const url_params = new URLSearchParams(location.search.slice(1));

		// Navbar buttons

		let dashboard_listener = document.createElement('div');
		dashboard_listener.className = 'button';
		dashboard_listener.innerText = 'Dashboard';
		core.navbar.append_left_html(dashboard_listener);

		dashboard_listener.addEventListener('click', () => core.open_view(this.parent != null && this.parent instanceof DashboardView ? this.parent : new DashboardView()));

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