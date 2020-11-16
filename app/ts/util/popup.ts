import { createElement } from './html';

export class Popup {
	container = createElement('div', { className: 'popup-container' });
	inner_container = createElement('div', { className: 'popup' }, this.container);

	constructor() {
		this.render();
	}

	render() {
		this.container.addEventListener('click', e => {
			if ((<HTMLElement>e.target) == this.container) {
				this.close();
			}
		});
	}

	open() {
		document.body.appendChild(this.container);
	}

	close() {
		this.container.remove();
	}
}


export function newEmptyPopup(): Popup {
	return new Popup();
}