import { Component, ElementRef, ViewChild } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';
import { WebsocketService } from 'src/app/websocket.service';

@Component({
	selector: 'app-editor',
	templateUrl: './editor.component.html',
	styleUrls: ['./editor.component.scss']
})

export class EditorComponent {
	constructor(public background: BackgroundService, public websocket: WebsocketService) {}

	ITEMS: string[] = [ 'title', 'link', 'guid', 'date', 'author', 'content' ];

	editing: ModelCustomItem = this.defaultEditing();

	displaying: { [name: string]: NodeContainer[] } = {};

	preview_url: string = 'https://google.com';

	@ViewChild('frame', { static: true, read: ElementRef })
	public iframeElement!: ElementRef<HTMLIFrameElement>;

	async save() {
		console.log(this.editing);
		this.websocket.send_new_custom_item(this.editing).then(console.log, console.error);
	}

	async test() {
		// this.websocket.send_test_watcher();
	}

	get frameWindow() {
		return this.iframeElement.nativeElement.contentWindow;
	}

	async previewIframe() {
		this.previewUnsetAll();

		try {
			let resp = await this.websocket.send_get_webpage_source(this.preview_url)

			if (this.frameWindow != null) {
				const iframe_doc = this.frameWindow.document;

				// Write webpage to iframe document
				iframe_doc.write(resp.html);

				let style = document.createElement('style');
				style.innerText = CUSTOM_IFRAME_CSS;
				iframe_doc.body.appendChild(style);
			}
		} catch(e) {
			console.error(e);
		}
	}

	previewUpdateMain() {
		this.previewUnsetAll();

		if (this.frameWindow != null && (this.editing.search_opts.items as string).length != 0) {
			let iframe_doc = this.frameWindow.document;

			try {
				let val = iframe_doc.evaluate(this.editing.search_opts.items as string, iframe_doc, null, XPathResult.ANY_TYPE, null);

				let items: Node[] = [];

				let last: Node | null = null;
				while ((last = val.iterateNext()) != null) {
					items.push(last);
				}

				items.forEach(i => (<HTMLElement>i).classList.add(getFrameColor('items')));

				this.displaying.items = items.map(i => new NodeContainer(i));

				this.previewUpdateChildren();

				console.log(`Found: ${items.length}`);
				// this.search_found.innerText = `Found: ${items.length}`;
			} catch(e) {
				console.error(e);
				// this.search_found.innerText = e;
			}
		// } else {
		// 	this.search_found.innerText = 'Found: 0';
		}
	}

	previewUpdateChildren(child_name?: string): void {
		if (!child_name) return this.ITEMS.forEach(v => this.previewUpdateChildren(v));

		if (this.frameWindow == null) return;

		let comp_item = this.displaying[child_name];

		if (comp_item == null) {
			comp_item = this.displaying[child_name] = [];
		} else {
			comp_item.forEach(i => i.getElement().classList.remove(getFrameColor(child_name)));
			comp_item.splice(0, comp_item.length);
		}

		if (this.editing.search_opts[child_name] == null || (this.editing.search_opts[child_name] as any).xpath.length == 0) {
			// this.search_found.innerText = '';
			return;
		}

		let search_opts_item = this.editing.search_opts[child_name] as { xpath: string; parse_type: rust.EnumValue; };

		try {
			this.displaying.items
			.forEach(context => {
				let context_element = context.getElement();

				let val = this.frameWindow!.document.evaluate(search_opts_item.xpath, context_element, null, XPathResult.ORDERED_NODE_ITERATOR_TYPE, null);

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

				comp_item.push(...items);
			});

			comp_item.forEach(i => i.getElement().classList.add(getFrameColor(child_name)));

			console.log(`Found: ${comp_item.length}`);
			// this.search_found.innerText = `Found: ${comp_item.length}`;
		} catch(e) {
			console.error(e);
			// this.search_found.innerText = e;
		}
	}

	previewUnsetAll() {
		for (let item in this.displaying) {
			if (this.displaying.hasOwnProperty(item)) {
				let found = this.displaying[item];
				found.forEach(i => i.getElement().classList.remove(getFrameColor(item)));
				delete this.displaying[item];
			}
		}
	}

	setSearchItemXpath(name: string, xpath: string) {
		if (this.editing.search_opts[name]) {
			(this.editing.search_opts[name] as any).xpath = xpath;
		} else {
			this.editing.search_opts[name] = {
				xpath: xpath,
				parse_type: 'None'
			};
		}
	}

	setSearchItemParser(name: string, parse_name: string) {
		if (this.editing.search_opts[name]) {
			(this.editing.search_opts[name] as any).parse_type = defaultParserType(parse_name);
		} else {
			this.editing.search_opts[name] = {
				xpath: '',
				parse_type: defaultParserType(parse_name)
			};
		}
	}

	getSearchItemParser(name: string) {
		if (this.editing.search_opts[name]) {
			let parse_type = (this.editing.search_opts[name] as any).parse_type;

			if (parse_type) {
				if (typeof parse_type == 'string') {
					return { name: parse_type, value: null };
				} else {
					parse_type = Object.entries(parse_type)[0];

					return {
						name: parse_type[0],
						value: parse_type[1]
					};
				}
			} else {
				return {
					name: 'None',
					value: null
				};
			}
		} else {
			return {
				name: 'None',
				value: null
			};
		}
	}

	setEditing(value: ModelCustomItem) {
		this.editing = value;
	}

	defaultEditing(): ModelCustomItem {
		return {
			title: '',
			description: '',
			match_url: '',
			search_opts: {}
		};
	}
}

function defaultParserType(name: string): rust.EnumValue {
	switch (name) {
		case 'None': return 'None'
		case 'TimeFormat': return { "TimeFormat": [ "", "" ] }
		case 'Regex': return { "Regex": "" }
		default: throw `Invalid Default Parser Type "${name}"`;
	}
}

function getFrameColor(name: string) {
	switch (name) {
		case 'items': return 'editor-border-items';
		case 'title': return 'editor-border-title';
		case 'link': return 'editor-border-link';
		case 'guid': return 'editor-border-guid';
		case 'date': return 'editor-border-date';
		case 'author': return 'editor-border-author';
		case 'content': return 'editor-border-content';

		default: throw new Error(`Not Frame Color for ${name}`);
	}
}

const CUSTOM_IFRAME_CSS = `
	.editor-border-items {
		border: #A00 1px solid;
	}

	.editor-border-title {
		border: #0A0 1px solid;
	}

	.editor-border-link {
		border: #00A 1px solid;
	}

	.editor-border-guid {
		border: #AA0 1px solid;
	}

	.editor-border-date {
		border: #A0A 1px solid;
	}

	.editor-border-author {
		border: #0AA 1px solid;
	}

	.editor-border-content {
		border: #A50 1px solid;
	}
`;


class NodeContainer {
	private node: Node

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