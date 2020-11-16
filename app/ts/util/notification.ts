import { createElement } from './html';

let container = createElement('div', { className: 'notification-container' });


interface NotificationOptions {
	description?: string;
	color?: 'red' | 'green' | 'yellow';

	// Amount of time to display it for.
	// <= 0 forever; > 0 time in ms.
	timer?: number;

	// The cause of the notification.
	cause?: string;

	// In MS
	created_at?: number;
}

const DEFAULT_DISPLAY_TIME = 1000 * 8;


function getOrCreateContainer() {
	if (container.parentElement == null) {
		document.body.appendChild(container);
	}

	return container;
}

class Notification {
	title: string;
	opts: NotificationOptions;

	timeout: Nullable<number> = null;

	container = createElement('div', { className: 'notification' });

	constructor(title: string, opts?: NotificationOptions) {
		this.title = title;
		this.opts = opts || {};

		if (this.opts.color) {
			this.container.classList.add(this.opts.color);
		}
	}

	display() {
		while(this.container.firstChild != null) {
			this.container.firstChild.remove();
		}

		let self = this;

		// Title Bar
		this.container.appendChild((function() {
			let inner = createElement('div', { className: 'flex-row title-bar' });

			// Title
			createElement('span', { className: 'title', innerText: self.title }, inner);

			// Button
			createElement('span', { className: 'close-button', innerText: 'X' }, inner)
			.addEventListener('click', () => self.close());

			return inner;
		}()));

		// Expand Info
		this.container.appendChild((function() {
			let inner = createElement('div', { className: 'flex-column expansion' });
			container.addEventListener('click', () => inner.classList.contains('show') ? inner.classList.remove('show') : inner.classList.add('show'));

			if (self.opts.description) {
				createElement('span', { className: 'desc', innerText: self.opts.description },  inner);
			}

			if (self.opts.cause) {
				createElement('span', { className: 'cause', innerText: self.opts.cause },  inner);
			}

			if (self.opts.created_at) {
				createElement('span', { className: 'creation', innerText: new Date(self.opts.created_at).toLocaleTimeString() },  inner);
			}

			return inner;
		}()));

		if (this.timeout == null) {
			let notif_container = getOrCreateContainer();
			notif_container.appendChild(this.container);
		} else {
			clearTimeout(this.timeout);
		}

		if (self.opts.timer == null) {
			this.timeout = setTimeout(() => this.close(), DEFAULT_DISPLAY_TIME);
		} else if (self.opts.timer > 0) {
			this.timeout = setTimeout(() => this.close(), self.opts.timer);
		}
	}

	close() {
		let notif_container = getOrCreateContainer();

		if (this.timeout != null) {
			clearTimeout(this.timeout);
			this.timeout = null;
		}

		this.container.remove();

		if (notif_container.children.length == 0) {
			notif_container.remove();
		}
	}
}


export function notify(title: string, opts?: NotificationOptions) {
	let notif = new Notification(title, opts);

	notif.display();

	return notif;
}

export function notifyError(title: string, opts?: Exclude<NotificationOptions, 'color'>) {
	return notify(title, Object.assign(opts || {}, { color: 'red' }));
}

export function notifySuccess(title: string, opts?: Exclude<NotificationOptions, 'color'>) {
	return notify(title, Object.assign(opts || {}, { color: 'green' }));
}

export function notifyWarning(title: string, opts?: Exclude<NotificationOptions, 'color'>) {
	return notify(title, Object.assign(opts || {}, { color: 'yellow' }));
}



export function notifyErrorDesc(title: string, description: string) {
	return notifyError(title, { description, timer: 0 });
}