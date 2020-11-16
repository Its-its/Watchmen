import SocketManager from './socket';
import BackgroundProcess from './process';
import Navbar from './navbar';

import View from './views/index';

import { notifyErrorDesc } from './util/notification';

// Views
import DashboardView from './views/dashboard';

import FeedEditorView from './views/feed/editor';
import FeedView from './views/feed/feeds';
import FeedFilterView from './views/feed/filter';
import FeedItemsView from './views/feed/items';

import WatcherEditorView from './views/watch/editor';
import WatcherItemsView from './views/watch/items';

function paths() {
	return {
		[DashboardView.path]: DashboardView,
		[FeedEditorView.path]: FeedEditorView,
		[FeedView.path]: FeedView,
		[FeedFilterView.path]: FeedFilterView,
		[FeedItemsView.path]: FeedItemsView,
		[WatcherEditorView.path]: WatcherEditorView,
		[WatcherItemsView.path]: WatcherItemsView,
	};
}

const app = {
	view: <Nullable<View>>null,

	socket: new SocketManager(),
	process: new BackgroundProcess(),
	navbar: new Navbar(),

	// Initiation

	init() {
		this.navbar.render();

		const url_params = new URLSearchParams(location.search.slice(1));

		if (url_params.has('view')) {
			let View = paths()[url_params.get('view')!];

			if (View != null) {
				return this.open_view(new View());
			}
		}

		this.open_view(new DashboardView());
	},

	on_connection_open() {
		// Get Current feeds
		this.process.init_feeds()
		.then(_ => {
			if (this.view != null) this.view.on_connection_open();
		})
		.catch(e => notifyErrorDesc('Initiate Feeds', e));

		this.process.register_updates()
		.catch(e => notifyErrorDesc('Initiate Feeds', e));
	},

	open_view(newView: View) {
		if (this.view != null) {
			this.view.close();
		}

		const url_params = new URLSearchParams(location.search.slice(1));
		url_params.set('view', newView.path);

		if (newView.path.length == 0) {
			url_params.delete('view');
		}

		let str = url_params.toString();

		window.history.replaceState(null, 'Page Change', str.length == 0 ? '/' : `/?${str}`);

		this.view = newView;

		this.view.init();
		this.view.open();
	},

	close_view() {
		if (this.view != null) {
			this.view.close();
			this.view = null;
		}
	}
};

// Ensures the imports are registered.
setTimeout(() => app.init(), 100);

// Utils
export function create_popup(cb: (container: HTMLDivElement, open: () => any, close: () => any) => any) {
	let container = document.createElement('div');
	container.className = 'popup-container';

	function close() {
		container.parentElement!.removeChild(container);
	}

	container.addEventListener('click', e => {
		if ((<HTMLElement>e.target) == container) {
			close()
		}
	});

	let inner = document.createElement('div');
	inner.className = 'popup';
	container.appendChild(inner);

	cb(inner, () => document.body.appendChild(container), close);
}

export function for_each<I, R>(items: I[], next_item: (item: I, item_finished: (...items: R[]) => any) => any, on_fin?: (resp: R[][]) => any) {
	let pos = 0;
	let finished: R[][] = [];

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


export default app;