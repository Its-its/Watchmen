

export default class View {
	parent = <Nullable<View>>null;

	initiated = false;
	container = document.createElement('div');

	constructor() {
		this.container.className = 'main-container';
	}


	public init() {
		if (!this.initiated) {
			this.on_init();
			this.initiated = true;
		}
	}

	public open() {
		if (this.container.parentElement == null) {
			document.body.appendChild(this.container);
			this.on_open();
		}
	}

	public close() {
		if (this.container.parentElement != null) {
			this.container.remove();
			this.on_close();
		}
	}


	public on_init() {}
	public on_open() {}
	public on_close() {}
}
