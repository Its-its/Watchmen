import core from '../core';

export default class View {
	path: string;

	initiated = false;
	container = document.createElement('div');

	constructor(path: string) {
		this.path = path;

		this.container.className = 'main-container';
	}


	public init() {
		console.log('View.init()');

		if (!this.initiated) {
			this.on_init();
			this.initiated = true;

			if (core.socket.is_open()) {
				console.log(' - Opening socket.');
				this.on_connection_open();
			}
		}
	}

	public open() {
		console.log('View.open()');

		if (this.container.parentElement == null) {
			document.body.appendChild(this.container);
			this.on_open();
		}
	}

	public close() {
		console.log('View.close()');

		if (this.container.parentElement != null) {
			this.container.remove();
			this.on_close();
		}
	}


	public on_init() {}
	public on_open() {}
	public on_close() {}

	public on_connection_open() {}

	static path = '';
}