import SocketManager from './socket';
import BackgroundProcess from './process';
import Navbar from './navbar';

import View from './views/index';
import DashboardView from './views/dashboard';

const app = {
	view: <Nullable<View>>null,

	socket: new SocketManager(),
	process: new BackgroundProcess(),
	navbar: new Navbar(),

	// Initiation

	init() {
		this.navbar.render();
		this.open_view(new DashboardView());
	},

	on_connection_open() {
		// Get Current feeds
		setTimeout(() => {
			this.process.refresh_feeds();
			this.process.register_updates();
		}, 50);
	},

	open_view(newView: View) {
		if (this.view != null) {
			this.view.close();

			if (newView.parent == null) {
				newView.parent = this.view;
			}
		}

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