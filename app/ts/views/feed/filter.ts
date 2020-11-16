import View from '../index';
import EditorView from './editor';
import FeedView from './items';

import core, { create_popup, for_each } from '../../core';

import { RustEnum } from '../../util/rust';
import { notifyErrorDesc } from '../../util/notification';

import {
	send_get_filter_list,
	send_update_filter,
	send_new_filter
} from '../../socket';


export default class FilterView extends View {
	main_filter = new FilterContainer();

	filter_container = document.createElement('div');
	nav_bar_list = document.createElement('ul');

	static path = 'feeds-edit-filter';

	constructor() {
		super(FilterView.path);
	}

	on_init() {
		this.render_sidebar();
		this.render_editor();
	}

	on_open() {
		// Navbar buttons
		let open_editor = document.createElement('div');
		open_editor.className = 'button';
		open_editor.innerText = 'Feed Viewer';
		core.navbar.append_left_html(open_editor);

		open_editor.addEventListener('click', () => core.open_view(new FeedView()));
	}

	on_close() {
		core.navbar.clear();
	}

	render_sidebar() {
		const nav_bar = document.createElement('div');
		nav_bar.className = 'nav-bar';

		const title_container = document.createElement('div');
		title_container.className = 'title-container';
		nav_bar.appendChild(title_container);

		const title = document.createElement('h1');
		title.className = 'title';
		title.innerText = 'Watchmen';
		title_container.appendChild(title);

		const nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		nav_bar.appendChild(nav_items);

		this.nav_bar_list.className = 'tree';
		nav_items.appendChild(this.nav_bar_list);

		this.container.appendChild(nav_bar);

		send_get_filter_list()
		.then(filterList => filterList.items.map(item => this.addItemToSidebar(item)))
		.catch(e => notifyErrorDesc('Grabbing Filter List', e));
	}

	render_editor() {
		this.filter_container.className = 'filter-container';
		this.container.appendChild(this.filter_container);

		this.display_filter(this.main_filter);
	}

	display_filter(cont: FilterContainer) {
		while(this.filter_container.firstChild) this.filter_container.firstChild.remove();

		this.main_filter = cont;

		// Tools

		const tool_bar = document.createElement('div');
		tool_bar.className = 'filter-tools';
		this.filter_container.appendChild(tool_bar);

		const title = document.createElement('input');
		title.placeholder = 'Filter Title.';
		title.type = 'text';
		tool_bar.appendChild(title);
		title.value = this.main_filter.title ? this.main_filter.title : '';
		title.addEventListener('keyup', () => this.main_filter.title = title.value);

		// NEW Filter.
		if (this.main_filter.filterId == null) {
			tool_bar.appendChild(renderButton('Create', () => {
				send_new_filter(this.main_filter.title!, this.main_filter.filter!.rust_enum.toJSON())
				.then(console.log)
				.catch(e => notifyErrorDesc('Creating New Filter', e));;

				console.log('Create');
			}));
		} else {
			tool_bar.appendChild(renderButton('Update', () => {
				console.log('Update');

				send_update_filter(
					this.main_filter.filterId!,
					this.main_filter.title!,
					this.main_filter.filter!.rust_enum.toJSON()
				)
				.then(console.log)
				.catch(e => notifyErrorDesc('Updating Existing Filter', e));;
			}));

			tool_bar.appendChild(renderButton('Clone', () => {
				console.log('Clone');
			}));
		}

		// Container

		const filterContainer = document.createElement('div');
		filterContainer.className = 'filter-editor';
		this.filter_container.appendChild(filterContainer);
		filterContainer.appendChild(this.main_filter.render());

		function renderButton(name: string, onClick: () => any) {
			const button = document.createElement('div');
			button.className = 'button';
			button.innerText = name;
			button.addEventListener('click', onClick);
			return button;
		}
	}

	addItemToSidebar(group: FilterGroupListener) {
		const item = document.createElement('div');
		item.className = 'tree-item';

		const title = document.createElement('div');
		title.className = 'tree-item-title';
		title.innerText = group.filter.title;
		item.appendChild(title);

		item.addEventListener('click', () => {
			this.display_filter(new FilterContainer(
				group.filter.id,
				group.filter.title,
				new Filter(new RustEnum(group.filter.filter))
			));
		})

		this.nav_bar_list.appendChild(item);
	}
}

class FilterContainer {
	title?: string;
	filterId?: number;

	filter: Nullable<Filter> = null;

	container = document.createElement('div');

	onBackEvent: Nullable<(filter: Filter) => any> = null;
	onSelectEvent: Nullable<(filter: Filter) => any> = null;

	constructor(filterId?: number, title?: string, filter?: Filter) {
		this.filterId = filterId;
		this.title = title;
		this.filter = filter ? filter : null;
	}

	render() {
		this.container.className = 'sub-filter';

		this.container.appendChild(this.render_select());

		return this.container;
	}

	render_select() {
		const container = document.createElement('div');
		container.className = 'filter filter-select';

		// Title
		const titleElement = document.createElement('h3');
		titleElement.className = 'title';
		titleElement.innerText = 'Select a Filter.';
		container.appendChild(titleElement);


		const content = document.createElement('div');
		content.className = 'content';
		container.appendChild(content);

		// Selection

		const back = () => {
			while (this.container.firstChild != null) this.container.firstChild.remove();
			this.container.appendChild(container);

			this.onBackEvent && this.onBackEvent(this.filter!);

			this.filter = null;
		}

		const renderOption = (name: string, render: (back: () => any) => HTMLElement) => {
			const option = document.createElement('option');
			option.innerText = name;

			option.addEventListener('click', () => {
				container.remove();
				this.container.appendChild(render.call(this, back));

				this.onSelectEvent && this.onSelectEvent(this.filter!);
			})

			return option;
		}

		const select = document.createElement('select');
		select.className = 'custom';

		const none = document.createElement('option');
		none.disabled = true;
		none.selected = true;
		none.innerText = 'Select One';
		select.appendChild(none);

		select.appendChild(renderOption('Regex', this.render_regex));
		select.appendChild(renderOption('Contains', this.render_contains));
		select.appendChild(renderOption('Starts With', this.render_starts_with));
		select.appendChild(renderOption('Ends With', this.render_ends_with));
		select.appendChild(renderOption('And Operation', this.render_and));
		select.appendChild(renderOption('Or Operation', this.render_or));
		content.appendChild(select);


		if (this.filter != null) {
			container.remove();

			this.onSelectEvent && this.onSelectEvent(this.filter);

			if (this.filter.is_rejex()) return this.render_regex(back);
			else if (this.filter.is_contains()) return this.render_contains(back);
			else if (this.filter.is_starts_with()) return this.render_starts_with(back);
			else if (this.filter.is_ends_with()) return this.render_ends_with(back);
			else if (this.filter.is_and()) return this.render_and(back);
			else if (this.filter.is_or()) return this.render_or(back);
		}

		return container;
	}

	render_regex(back: () => any) {
		if (this.filter == null) this.filter = new_regex_filter();

		const container = document.createElement('div');
		container.className = 'filter filter-regex';

		// Title
		const titleContainer = document.createElement('div');
		titleContainer.className = 'title';
		container.appendChild(titleContainer);

		const removeSelf = document.createElement('span');
		removeSelf.className = 'delete';
		removeSelf.innerText = 'X';
		titleContainer.appendChild(removeSelf);

		const titleElement = document.createElement('span');
		titleElement.innerText = 'Regex Filter';
		titleContainer.appendChild(titleElement);

		removeSelf.addEventListener('click', () => back());


		const content = document.createElement('div');
		content.className = 'content';
		container.appendChild(content);


		// Input
		const inputContainer = document.createElement('div');
		inputContainer.className = 'grouping';
		content.appendChild(inputContainer);

		const inputLabel = document.createElement('h4');
		inputLabel.className = 'sub-title';
		inputLabel.innerText = 'Regex Query';
		inputContainer.appendChild(inputLabel);

		const inputRegex = document.createElement('input');
		inputRegex.type = 'text';
		inputRegex.placeholder = 'Regex Query';
		inputContainer.appendChild(inputRegex);

		// @ts-ignore
		inputRegex.addEventListener('keyup', () => this.filter!.rust_enum.value[0] = inputRegex.value);

		// Regex Opts
		const regexOptsContainer = document.createElement('div');
		regexOptsContainer.className = 'grouping';
		content.appendChild(regexOptsContainer);

		const regexOptsLabel = document.createElement('h4');
		regexOptsLabel.className = 'sub-title';
		regexOptsLabel.innerText = 'Regex Options';
		regexOptsContainer.appendChild(regexOptsLabel);

		const regexOpts = document.createElement('select');
		regexOpts.className = 'custom';
		regexOpts.multiple = true;

		const appendOption = (title: string, value: string, defSelection: boolean) => {
			const option = document.createElement('option');
			option.innerText = title;
			option.value = value;
			// @ts-ignore
			option.selected = this.filter!.rust_enum.value[1] != null ? this.filter!.rust_enum.value[1][value] == true : defSelection;

			regexOpts.appendChild(option);
		};

		appendOption('Dot Matches New Line', 'dot_matches_new_line', false);
		appendOption('Ingore Whitespace', 'ignore_whitespace', false);
		appendOption('Case Insensitive', 'case_insensitive', true);
		appendOption('Multi Line', 'multi_line', false);
		appendOption('Swap Greed', 'swap_greed', false);
		appendOption('Unicode', 'unicode', true);
		appendOption('Octal', 'octal', false);

		regexOptsContainer.appendChild(regexOpts);

		const publishRegxOpts = () => {
			let values: { [key: string]: boolean } = {};

			for (let i = 0; i < regexOpts.options.length; i++) {
				const option = regexOpts.options[i];
				values[option.value] = option.selected;
			}

			// @ts-ignore
			this.filter!.rust_enum.value[1] = values;
		}

		regexOpts.addEventListener('change', () => publishRegxOpts());

		// @ts-ignore
		inputRegex.value = this.filter!.rust_enum.value[0];

		// @ts-ignore
		if (this.filter!.rust_enum.value[1] == null) {
			publishRegxOpts();
		}

		return container;
	}

	render_contains(back: () => any) {
		if (this.filter == null) this.filter = new_contains_filter();
		return this.render_multi('Contains', back);
	}

	render_starts_with(back: () => any) {
		if (this.filter == null) this.filter = new_starts_with_filter();
		return this.render_multi('Starts With', back);
	}

	render_ends_with(back: () => any) {
		if (this.filter == null) this.filter = new_ends_with_filter();
		return this.render_multi('Ends With', back);
	}

	render_or(back: () => any) {
		if (this.filter == null) this.filter = new_or_filter();
		return this.render_multi_and_or('OR', back);
	}

	render_and(back: () => any) {
		if (this.filter == null) this.filter = new_and_filter();
		return this.render_multi_and_or('AND', back);
	}

	render_multi_and_or(title: string, back: () => any) {
		const container = document.createElement('div');
		container.className = `filter filter-${title.toLowerCase().replace(/ /g, '-')}`;


		// Title
		const titleContainer = document.createElement('div');
		titleContainer.className = 'title';
		container.appendChild(titleContainer);

		const removeSelf = document.createElement('span');
		removeSelf.className = 'delete';
		removeSelf.innerText = 'X';
		titleContainer.appendChild(removeSelf);

		const titleElement = document.createElement('span');
		titleElement.innerText = title + ' Match Filters';
		titleContainer.appendChild(titleElement);

		removeSelf.addEventListener('click', () => back());


		// Content
		const filterContent = document.createElement('div');
		filterContent.className = 'content';
		container.appendChild(filterContent);

		const addFilterContainer = (filter?: Filter) => {
			let newFilter = new FilterContainer(undefined, undefined, filter);

			let itemContainer = newFilter.render();

			filterContent.appendChild(itemContainer);

			newFilter.onBackEvent = (prevFilter) => {
				itemContainer.remove();

				let indexOf = (<RustEnum[]>this.filter!.rust_enum.value).indexOf(prevFilter.rust_enum);
				(<RustEnum[]>this.filter!.rust_enum.value).splice(indexOf, 1);
			};

			newFilter.onSelectEvent = (newFilter) => {
				addFilterContainer();

				(<RustEnum[]>this.filter!.rust_enum.value).push(newFilter.rust_enum);
			};
		};

		(<RustEnum[]>this.filter!.rust_enum.value)
		.forEach(e => addFilterContainer(new Filter(e)));

		addFilterContainer();

		return container;
	}


	render_multi(title: string, back: () => any) {
		const container = document.createElement('div');
		container.className = `filter filter-${title.toLowerCase().replace(/ /g, '-')}`;

		// Title
		const titleContainer = document.createElement('div');
		titleContainer.className = 'title';
		container.appendChild(titleContainer);

		const removeSelf = document.createElement('span');
		removeSelf.className = 'delete';
		removeSelf.innerText = 'X';
		titleContainer.appendChild(removeSelf);

		const titleElement = document.createElement('span');
		titleElement.innerText = title + ' Filter';
		titleContainer.appendChild(titleElement);

		removeSelf.addEventListener('click', () => back());

		// Content

		const content = document.createElement('div');
		content.className = 'content';
		container.appendChild(content);

		// Input
		const inputContainer = document.createElement('div');
		inputContainer.className = 'grouping';
		content.appendChild(inputContainer);

		const inputLabel = document.createElement('h4');
		inputLabel.className = 'sub-title';
		inputLabel.innerText = 'Find Value';
		inputContainer.appendChild(inputLabel);

		const inputRegex = document.createElement('input');
		inputRegex.type = 'text';
		inputRegex.placeholder = 'Value';
		inputContainer.appendChild(inputRegex);

		// @ts-ignore
		inputRegex.addEventListener('keyup', () => this.filter!.rust_enum.value[0] = inputRegex.value);


		// Case Sensitive
		const csContainer = document.createElement('div');
		csContainer.className = 'grouping';
		content.appendChild(csContainer);

		const csLabel = document.createElement('h4');
		csLabel.className = 'sub-title';
		csLabel.innerText = 'Case Sensitive';
		csContainer.appendChild(csLabel);

		const csCheck = document.createElement('input');
		csCheck.type = 'checkbox';
		csCheck.className = 'custom';
		csContainer.appendChild(csCheck);

		// @ts-ignore
		csCheck.addEventListener('change', () => this.filter!.rust_enum.value[1] = csCheck.checked);

		// @ts-ignore
		inputRegex.value = this.filter!.rust_enum.value[0];
		// @ts-ignore
		csCheck.checked = this.filter!.rust_enum.value[1];

		return container;
	}
}

class Filter {
	rust_enum: RustEnum;

	constructor(rust_enum: RustEnum) {
		this.rust_enum = rust_enum;
	}

	is_rejex() {
		return this.rust_enum.name == 'Regex';
	}

	is_contains() {
		return this.rust_enum.name == 'Contains';
	}

	is_starts_with() {
		return this.rust_enum.name == 'StartsWith';
	}

	is_ends_with() {
		return this.rust_enum.name == 'EndsWith';
	}

	is_and() {
		return this.rust_enum.name == 'And';
	}

	is_or() {
		return this.rust_enum.name == 'Or';
	}
}

function new_regex_filter() {
	return new Filter(new RustEnum("Regex", ['', null]));
}

function new_contains_filter() {
	return new Filter(new RustEnum("Contains", ['', false]));
}

function new_starts_with_filter() {
	return new Filter(new RustEnum("StartsWith", ['', false]));
}

function new_ends_with_filter() {
	return new Filter(new RustEnum("EndsWith", ['', false]));
}

function new_and_filter() {
	return new Filter(new RustEnum("And", []));
}

function new_or_filter() {
	return new Filter(new RustEnum("Or", []));
}