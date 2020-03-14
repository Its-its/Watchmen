


export default class Navbar {
	container = document.getElementById('top-nav-bar')!;

	nav_left = document.createElement('div');
	nav_right = document.createElement('div');

	render() {
		this.nav_left.className = 'nav-container left';
		this.container.appendChild(this.nav_left);

		this.nav_right.className = 'nav-container right';
		this.container.appendChild(this.nav_right);
	}


	clear() {
		while (this.nav_left.firstChild) this.nav_left.firstChild.remove();
		while (this.nav_right.firstChild) this.nav_right.firstChild.remove();
	}

	append_left_html(element: HTMLElement) {
		this.nav_left.appendChild(element);
	}

	append_right_html(element: HTMLElement) {
		this.nav_right.appendChild(element);
	}
}