import core from '../core';
import { rustify_object, RustEnum, NULL_ENUM } from '../util/rust';
import { parseFromString } from '../util/time';

import View from './index';
import FeedView from './feed';

import { send_get_webpage_source, send_new_custom_item, send_get_custom_items_list } from '../socket';


type ItemTypes = 'items' | 'title' | 'link' | 'guid' | 'date' | 'author' | 'content';
type TypeConf = {
	found: NodeContainer[],
	xpath: Nullable<string>,
	parseType: RustEnum,
	invalid: boolean
};


export default class EditorView extends View {
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

		open_editor.addEventListener('click', () => {
			if (this.parent != null) {
				core.open_view(this.parent);
			} else {
				core.open_view(new FeedView());
			}
		});
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
		title.innerText = 'Feeder';
		title_container.appendChild(title);

		const nav_items = document.createElement('div');
		nav_items.className = 'nav-bar-items';
		nav_bar.appendChild(nav_items);

		const nav_bar_list = document.createElement('ul');
		nav_bar_list.className = 'tree';
		nav_items.appendChild(nav_bar_list);

		this.container.appendChild(nav_bar);
	}

	render_editor() {
		const container = document.createElement('div');
		container.className = 'editor-container';
		this.container.appendChild(container);

		const iframe_container = document.createElement('div');
		iframe_container.className = 'window';
		container.appendChild(iframe_container);

		const iframe = document.createElement('iframe');
		iframe.className = 'frame';
		iframe.setAttribute('sandbox', 'allow-same-origin');
		iframe_container.appendChild(iframe);

		const xpath_items = document.createElement('div');
		container.appendChild(xpath_items);

		// Multiple Feed Items Selection

		const ITEMS: ItemTypes[] = [ 'items', 'title', 'link', 'guid', 'date', 'author', 'content' ];

		let compiled: {
			[name: string]: TypeConf;
		} = {};

		ITEMS.forEach(i => {
			compiled[i] = {
				found: [],
				xpath: null,
				parseType: NULL_ENUM(),
				invalid: false
			};
		});

		function resetAll() {
			for (let item in compiled) {
				if (compiled.hasOwnProperty(item)) {
					let element = compiled[item];

					element.found.forEach(i => i.getElement().classList.remove('editor-border-red'));
					element.found = [];
					element.xpath = null;
					element.invalid = true;
				}
			}
		}

		// Custom Site Info
		let custom_cont = document.createElement('div');
		xpath_items.appendChild(custom_cont);


		const site_url = document.createElement('h3');
		site_url.style.marginTop = '5px';
		site_url.style.marginBottom = '0';
		site_url.innerText = 'Website URL';
		custom_cont.appendChild(site_url);

		let custom_url = document.createElement('input');
		custom_url.value = 'https://www.ebay.com/sch/i.html?_from=R40&_nkw=x-h1&_sacat=0&_sop=10';
		custom_url.placeholder = 'URL';
		custom_url.type = 'text';
		custom_cont.appendChild(custom_url);

		let custom_url_preview = document.createElement('div');
		custom_url_preview.className = 'button';
		custom_url_preview.innerText = 'Preview';
		custom_cont.appendChild(custom_url_preview);

		custom_url_preview.addEventListener('click', () => {
			resetAll();

			send_get_webpage_source(custom_url.value, (err, resp) => {
				if (iframe.contentWindow != null) {
					const iframe_doc = iframe.contentWindow.document;

					// Write webpage to iframe document
					iframe_doc.write(resp!.html);

					let style = document.createElement('style');
					style.innerText = CUSTOM_IFRAME_CSS;
					iframe_doc.body.appendChild(style);
				}
			});
		});


		const site_title = document.createElement('h3');
		site_title.style.marginTop = '5px';
		site_title.style.marginBottom = '0';
		site_title.innerText = 'Custom Site';
		custom_cont.appendChild(site_title);

		let custom_name = document.createElement('input');
		custom_name.placeholder = 'Custom Website Title';
		custom_name.type = 'text';
		custom_cont.appendChild(custom_name);

		custom_cont.appendChild(document.createElement('br'));

		let custom_desc = document.createElement('input');
		custom_desc.placeholder = 'Custom Website Desc.';
		custom_desc.type = 'text';
		custom_cont.appendChild(custom_desc);

		custom_cont.appendChild(document.createElement('br'));

		let custom_cont_url = document.createElement('input');
		custom_cont_url.placeholder = 'Contains URL';
		custom_cont_url.type = 'text';
		custom_cont.appendChild(custom_cont_url);

		let subItems = [
			new ItemInfoSearch('Title', 'title', iframe, compiled),
			new ItemInfoSearch('Link', 'link', iframe, compiled),
			new ItemInfoSearch('Unique ID', 'guid', iframe, compiled),
			new ItemInfoSearch('Upload Date', 'date', iframe, compiled),
			new ItemInfoSearch('Author', 'author', iframe, compiled),
			new ItemInfoSearch('HTML Content', 'content', iframe, compiled)
		];

		// Items Search
		let mainItem = new MainItemsSearch(subItems, iframe, compiled).render(xpath_items);


		// Item Info
		const item_info_cont = document.createElement('div');
		item_info_cont.setAttribute('style', 'margin-left: 40px;');
		xpath_items.appendChild(item_info_cont);

		// TODO: Cannot use same xpath on two different searches.

		subItems.forEach(i => i.render(item_info_cont));

		//
		const confirm_button = document.createElement('div');
		confirm_button.style.float = 'left';
		confirm_button.innerText = 'Update';
		confirm_button.className = 'button';
		container.appendChild(confirm_button);

		confirm_button.addEventListener('click', () => {
			let rustify: ModelCustomItem = {
				title: custom_name.value,
				description: custom_desc.value,
				match_url: custom_cont_url.value,

				search_opts: {
					items: compiled.items.xpath!
				}
			};

			for (let name in compiled) {
				if (compiled.hasOwnProperty(name) && name != 'items') {
					let values = compiled[name];

					// if (values.invalid) {
					// 	return console.log('Invalid:', values);
					// }

					if ((values.xpath == null || values.xpath == 'None') && values.parseType.name == 'None') {
						rustify.search_opts[name] = null;
					} else {
						rustify.search_opts[name] = {
							xpath: values.xpath!,
							parse_type: values.parseType
						};
					}
				}
			}

			console.log(JSON.stringify(rustify, null, 4));

			let obj = rustify_object(rustify);

			send_new_custom_item(obj, (err, value, method) => {
				console.log(err);
				console.log(method);
				console.log(value);
			});
		});


		load({
			"title": "Ebay",
			"description": "Ebay desc",
			"match_url": "ebay.com",
			"search_opts": {
				"items": "//ul[@class=\"srp-results srp-list clearfix\"]/li",
				"title": {
					"xpath": ".//h3/text()",
					"parse_type": "None"
				},
				"link": {
					"xpath": ".//a[@class=\"s-item__link\"]/@href",
					"parse_type": {
						"Regex": "^([a-z0-9:/.-]+)"
					}
				},
				"guid": {
					"xpath": ".//a[@class=\"s-item__link\"]/@href",
					"parse_type": {
						"Regex": "^[a-z0-9:/.-]+/([0-9]+)\?"
					}
				},
				"date": {
					"xpath": ".//span[@class=\"s-item__dynamic s-item__listingDate\"]/span/text()",
					"parse_type": {
						"TimeFormat": [ "%b-%e %R", "PST" ]
					}
				},
				"author": {
					"xpath": ".//span[@class=\"s-item__seller-info-text\"]/text()",
					"parse_type": "None"
				},
				"content": {
					"xpath": "./node()",
					"parse_type": "None"
				}
			}
		});

		function load(opts: any) {
			custom_name.value = opts.title;
			custom_desc.value = opts.description;
			custom_cont_url.value = opts.match_url;

			for (const key in opts.search_opts) {
				if (opts.search_opts.hasOwnProperty(key)) {
					const config = opts.search_opts[key];

					if (key == 'items') {
						mainItem.setValue(config);
					} else {
						let found = subItems.find(i => i.config_name == key);

						if (found != null) {
							found.load(config);
						}
					}
				}
			}

			send_get_webpage_source(custom_url.value, (err, resp) => {
				if (iframe.contentWindow != null) {
					const iframe_doc = iframe.contentWindow.document;

					// Write webpage to iframe document
					iframe_doc.write(resp!.html);

					let style = document.createElement('style');
					style.innerText = CUSTOM_IFRAME_CSS;
					iframe_doc.body.appendChild(style);

					setTimeout(() => mainItem.findItems(), 1000);
				}
			});
		}
	}
}

class NodeContainer {
	/** @private */
	node: Node

	constructor(node: Node) {
		this.node = node;
	}

	getElement(): HTMLElement {
		if (this.node instanceof Text) {
			return this.node.parentElement!;
		} else if (this.node instanceof Attr) {
			// @ts-ignore
			return this.node.ownerElement;
		} else {
			// @ts-ignore
			return this.node;
		}
	}

	value(): Nullable<string> {
		if (this.isText()) {
			return this.node.nodeValue;
		} else if (this.isAttribute()) {
			return this.node.nodeValue;
		} else {
			return null;
		}
	}

	isText(): boolean {
		return this.node instanceof Text;
	}

	isAttribute(): boolean {
		return this.node instanceof Attr;
	}
}


class ItemInfoSearch {
	title: string;
	config_name: string;
	iframe: HTMLIFrameElement;
	compiled: { [name: string]: TypeConf; };

	search_input = document.createElement('input');
	search_found = document.createElement('span');

	parser_container = document.createElement('div');
	foundItems = document.createElement('textarea');
	parse_type_selection = document.createElement('select');

	constructor(title: string, config_name: string, iframe: HTMLIFrameElement, compiled: { [name: string]: TypeConf; }) {
		this.title = title;
		this.config_name = config_name;
		this.iframe = iframe;
		this.compiled = compiled;
	}

	parseTypeName() {
		return this.compiled[this.config_name].parseType.name;
	}

	value() {
		return this.search_input.value;
	}

	setValue(value: string) {
		this.search_input.value = value;
		this.updateItemInfo();
	}

	setParseType(value: RustEnum) {
		// this.search_input.value = value;
		// this.updateItemInfo();
	}

	updateItemInfo() {
		let comp_item = this.compiled[this.config_name];

		comp_item.found.forEach(i => i.getElement().classList.remove('editor-border-red'));
		comp_item.found = [];

		comp_item.xpath = this.value();

		if (comp_item.xpath.length == 0) {
			this.search_found.innerText = '';
			comp_item.xpath = null;
			return;
		}

		try {
			this.compiled.items.found
			.forEach(context => {
				let context_element = context.getElement();

				let val = this.iframe.contentWindow!.document.evaluate(comp_item.xpath!, context_element, null, XPathResult.ORDERED_NODE_ITERATOR_TYPE, null);

				let find_multiple_items = false;

				let items: NodeContainer[] = [];

				let last: Nullable<Node> = null;
				while((last = val.iterateNext()) != null) {
					if (last == null) break;

					// Ensure it's not selecting the base element
					if (last == context_element) {
						console.log('Attempted to select base element');
						break;
					}

					let cont = new NodeContainer(last);

					items.push(cont);

					if (!find_multiple_items) {
						break;
					}

					if (items.length > 999) {
						console.log('possible overflow');
						break;
					}
				}

				comp_item.found = comp_item.found.concat(items);
			});

			comp_item.found.forEach(i => i.getElement().classList.add('editor-border-red'));

			this.search_found.innerText = `Found: ${comp_item.found.length}`;
		} catch(e) {
			this.search_found.innerText = e;
		}
	}


	load(opts?: { xpath: string, parse_type: rust.EnumObject } | rust.EnumNone) {
		if (opts == null || opts == "None") {
			this.setValue('');
			this.displayNoneType();
		} else {
			let parseType = new RustEnum(opts.parse_type);

			console.log(parseType);

			this.parser_container.childNodes.forEach(i => i.remove());
			this.parse_type_selection.value = parseType.name;

			switch (parseType.name) {
				case 'Regex':
					this.displayRegexType(parseType.value);
					break;
				case 'TimeFormat':
					this.displayTimeFormatType(parseType.value);
					break;
				case 'None':
					this.displayNoneType();
					break;
			}

			this.setValue(opts.xpath);
		}
	}


	// Display

	render(container: HTMLElement): ItemInfoSearch {
		let search_cont = document.createElement('div');
		search_cont.style.marginTop = '10px';
		container.appendChild(search_cont);

		let title = document.createElement('h3');
		title.style.marginTop = '5px';
		title.style.marginBottom = '0';
		title.innerText = this.title;
		search_cont.appendChild(title);

		this.search_input = document.createElement('input');
		this.search_input.id = 'set-config-' + this.config_name;
		this.search_input.placeholder = `${this.title} XPath`;
		this.search_input.type = 'text';
		search_cont.appendChild(this.search_input);

		this.search_found = document.createElement('span');
		search_cont.appendChild(this.search_found);

		this.createParser(search_cont);

		let last_timeout_id: Nullable<number> = null;

		this.search_input.addEventListener('mouseup', () => {
			if (last_timeout_id != null) {
				clearTimeout(last_timeout_id);
			}

			if (this.compiled.items.found.length == 0) return;

			last_timeout_id = setTimeout(() => {
				last_timeout_id = null;
				if (this.compiled.items.found.length == 0) return;

				this.updateItemInfo();
			}, 500);
		});

		return this;
	}

	findItems() {}

	createParser(parent: HTMLElement) {
		let container = document.createElement('div');
		container.style.marginTop = '10px';
		parent.appendChild(container);

		// Types
		const parsers = ['None', 'Regex', 'TimeFormat'];
		container.appendChild(this.parse_type_selection);
		parsers.forEach(name => {
			let option = document.createElement('option');
			option.value = name;
			option.innerText = name;

			if (name == 'None') option.defaultSelected = true;

			this.parse_type_selection.appendChild(option);
		});

		container.appendChild(this.parser_container);

		this.displayNoneType();

		this.parse_type_selection.addEventListener('change', () => {
			this.parser_container.childNodes.forEach(i => i.remove());

			switch (this.parse_type_selection.value) {
				case 'Regex':
					this.displayRegexType();
					break;
				case 'TimeFormat':
					this.displayTimeFormatType();
					break;

				default:
					this.displayNoneType();
					break;
			}
		});
	}

	displayNoneType() {
		let comp_item = this.compiled[this.config_name];

		comp_item.parseType.name = 'None';
		comp_item.parseType.value = null;

		let examples_toggled = false;
		let toggle_examples = document.createElement('span');
		toggle_examples.innerText = 'Show Found';
		toggle_examples.style.cursor = 'pointer';
		this.parser_container.appendChild(toggle_examples);

		this.foundItems.rows = 10;
		this.foundItems.style.display = 'none';
		this.foundItems.style.width = 'calc(100% - 20px)';
		this.parser_container.appendChild(this.foundItems);

		toggle_examples.addEventListener('click', () => {
			if (examples_toggled) {
				toggle_examples.innerText = 'Show Found';
				this.foundItems.style.display = 'none';
			} else {
				toggle_examples.innerText = 'Hide Found';
				this.foundItems.style.display = 'block';
			}

			examples_toggled = !examples_toggled;
		});
	}

	displayRegexType(value?: rust.EnumValue) {
		let comp_item = this.compiled[this.config_name];

		comp_item.parseType.name = 'Regex';
		comp_item.parseType.value = (value != null ? value : '');

		let input = document.createElement('input');
		// @ts-ignore
		if (value != null) input.value = value;
		input.type = 'text';
		input.placeholder = 'Regex';
		this.parser_container.appendChild(input);

		this.parser_container.appendChild(document.createElement('br'));

		let examples_toggled = false;
		let toggle_examples = document.createElement('span');
		toggle_examples.innerText = 'Show Conversions';
		toggle_examples.style.cursor = 'pointer';
		this.parser_container.appendChild(toggle_examples);

		toggle_examples.addEventListener('click', () => {
			if (examples_toggled) {
				toggle_examples.innerText = 'Hide Conversions';
				this.foundItems.style.display = 'block';
			} else {
				toggle_examples.innerText = 'Show Conversions';
				this.foundItems.style.display = 'none';
			}

			examples_toggled = !examples_toggled;
		});

		this.foundItems.rows = 10;
		this.foundItems.style.display = 'none';
		this.foundItems.style.width = 'calc(100% - 20px)';
		this.parser_container.appendChild(this.foundItems);

		input.addEventListener('mousedown', () => {
			comp_item.parseType.value = input.value;

			this.foundItems.value = '';

			if (comp_item.found.length != 0) {
				try {
					for (let i = 0; i < comp_item.found.length; i++){
						let found = comp_item.found[i];

						let regex = new RegExp(input.value, 'gi');
						let exec = regex.exec(found.value()!);

						if (exec == null) {
							this.foundItems.value = 'Not able to find any matches.';
							break;
						} else {
							this.foundItems.value += `[${i}]: ${exec[1]}\n`;
						}
					};
				} catch(e) {
					this.foundItems.value = e;
				}
			}
		});
	}

	displayTimeFormatType(value?: rust.EnumValue) {
		let comp_item = this.compiled[this.config_name];

		comp_item.parseType.name = 'TimeFormat';
		comp_item.parseType.value = [];

		let format_input = document.createElement('input');
		// @ts-ignore
		if (format_input != null) format_input.value = value[0];
		format_input.type = 'text';
		format_input.placeholder = 'Time Format';
		this.parser_container.appendChild(format_input);

		this.parser_container.appendChild(document.createElement('br'));

		let tz_input = document.createElement('input');
		// @ts-ignore
		if (tz_input != null) tz_input.value = value[1];
		tz_input.type = 'text';
		tz_input.placeholder = 'Timezone of the Date/Time (ex: UTC)';
		this.parser_container.appendChild(tz_input);

		this.parser_container.appendChild(document.createElement('br'));

		let examples_toggled = false;
		let toggle_examples = document.createElement('span');
		toggle_examples.innerText = 'Show Conversions';
		toggle_examples.style.cursor = 'pointer';
		this.parser_container.appendChild(toggle_examples);

		toggle_examples.addEventListener('click', () => {
			if (examples_toggled) {
				toggle_examples.innerText = 'Show Conversions';
				this.foundItems.style.display = 'none';
			} else {
				toggle_examples.innerText = 'Hide Conversions';
				this.foundItems.style.display = 'block';
			}

			examples_toggled = !examples_toggled;
		});

		this.foundItems.rows = 10;
		this.foundItems.style.display = 'none';
		this.foundItems.style.width = 'calc(100% - 20px)';
		this.parser_container.appendChild(this.foundItems);

		format_input.addEventListener('mousedown', onChange);
		tz_input.addEventListener('mousedown', onChange);

		let self = this;

		function onChange() {
			comp_item.invalid = true;
			if (tz_input.value.length == 0 || format_input.value.length == 0) return;

			let error = false;

			comp_item.parseType.value = [format_input.value, tz_input.value];

			self.foundItems.value = '';

			if (comp_item.found.length != 0) {
				for(let i = 0; i < comp_item.found.length; i++){
					let found = comp_item.found[i];

					let time = parseFromString(`${found.value()!} ${tz_input.value}`, `${format_input.value} %Z`);

					if (time == null) {
						self.foundItems.value = 'Unable to parse.';
						error = true;
						break;
					} else {
						self.foundItems.value += `[${i}]: ${time}\n`;
					}
				}
			}

			if (error == null) {
				comp_item.invalid = false;
			}
		}
	}
}


class MainItemsSearch extends ItemInfoSearch {
	subItems: ItemInfoSearch[];

	constructor(subItems: ItemInfoSearch[], iframe: HTMLIFrameElement, compiled: { [name: string]: TypeConf; }) {
		super('Items', 'items', iframe, compiled);
		this.subItems = subItems;
	}

	render(container: HTMLElement): ItemInfoSearch {
		let search_cont = document.createElement('div');
		container.appendChild(search_cont);

		let item_title = document.createElement('h3');
		item_title.style.marginTop = '5px';
		item_title.style.marginBottom = '0';
		item_title.innerText = 'Items';
		search_cont.appendChild(item_title);

		this.search_input = document.createElement('input');
		this.search_input.placeholder = 'Items Container (xpath)';
		this.search_input.type = 'text';
		search_cont.appendChild(this.search_input);

		this.search_found = document.createElement('span');
		search_cont.appendChild(this.search_found);

		let last_timeout_id: Nullable<number> = null;

		this.search_input.addEventListener('mouseup', () => {
			if (last_timeout_id != null) {
				clearTimeout(last_timeout_id);
			}

			last_timeout_id = setTimeout(() => {
				last_timeout_id = null;

				this.findItems();
			}, 500);
		});

		return this;
	}

	findItems() {
		this.compiled.items.found.forEach(i => i.getElement().classList.remove('editor-border-red'));
		this.unsetChildBorders();
		this.compiled.items.found = [];
		// TODO: Call reset child items 'found'

		this.compiled.items.xpath = this.search_input.value;

		if (this.search_input.value.length != 0) {
			let iframe_doc = this.iframe.contentWindow!.document;

			try {
				var val = iframe_doc.evaluate(this.search_input.value, iframe_doc, null, XPathResult.ANY_TYPE, null);

				var items: Node[] = [];

				var last: Node | null = null;
				while ((last = <any>val.iterateNext()) != null) {
					items.push(last);
				}

				items.forEach(i => (<HTMLElement>i).classList.add('editor-border-red'));

				this.compiled.items.found = items.map(i => new NodeContainer(i));

				// this.resetChildBorders();
				this.subItems.forEach(i => i.updateItemInfo());

				this.search_found.innerText = `Found: ${items.length}`;
			} catch(e) {
				this.search_found.innerText = e;
			}
		} else {
			this.search_found.innerText = 'Found: 0';
		}
	}

	unsetChildBorders() {
		for (let key in this.compiled) {
			if (this.compiled.hasOwnProperty(key)) {
				let element = this.compiled[key];

				if (key != 'items') {
					element.found.forEach(i => i.getElement().classList.remove('editor-border-red'));
				}
			}
		}
	}

	resetChildBorders() {
		for (let item in this.compiled) {
			if (this.compiled.hasOwnProperty(item)) {
				let element = this.compiled[item];

				if (item != 'items') {
					element.found.forEach(i => i.getElement().classList.add('editor-border-red'));
				}
			}
		}
	}
}


const CUSTOM_IFRAME_CSS = `
	.editor-border-red {
		border: #F00 1px solid;
	}
`;