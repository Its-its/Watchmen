import { createElement } from './html';

export class Popup {
	rendered = false;

	container = createElement('div', { className: 'popup-container' });
	inner_container = createElement('div', { className: 'popup' }, this.container);

	render() {
		if (this.rendered) return;
		this.rendered = true;

		this.container.addEventListener('click', e => {
			if ((<HTMLElement>e.target) == this.container) {
				this.close();
			}
		});
	}

	open() {
		this.render();

		document.body.appendChild(this.container);
	}

	close() {
		this.container.remove();
	}
}


type FormValue = HTMLInputElement | HTMLSelectElement | HTMLElement;

export class FormPopup extends Popup {
	form = createElement('form', { className: 'form-group' }, this.inner_container);

	form_values: FormValue[][];

	constructor(form_values: FormValue[][]) {
		super();

		this.form_values = form_values;
	}

	render() {
		if (this.rendered) return;
		super.render();

		for (let i = 0; i < this.form_values.length; i++) {
			const rowValue = this.form_values[i];

			const rowElement = createElement('div', { className: 'form-row' }, this.form);

			for (let x = 0; x < rowValue.length; x++) {
				rowElement.appendChild(rowValue[x]);
			}

			this.form.appendChild(rowElement);
		}

	}

	addRow(rowValue: FormValue | FormValue[]) {
		if (Array.isArray(rowValue)) {
			this.form_values.push(rowValue);
		} else {
			this.form_values.push([rowValue]);
		}
	}

	values() {
		//
	}
}


export function newEmptyPopup(): Popup {
	return new Popup();
}

export function newFormPopup(form_values?: FormValue[][]): FormPopup {
	return new FormPopup(form_values || []);
}