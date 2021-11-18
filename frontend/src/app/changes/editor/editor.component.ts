import { Component, ElementRef, ViewChild } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';
import { WebsocketService } from 'src/app/websocket.service';

@Component({
	selector: 'app-editor',
	templateUrl: './editor.component.html',
	styleUrls: ['./editor.component.scss']
})

export class ChangesEditorComponent {
	constructor(public background: BackgroundService, public websocket: WebsocketService) {}

	ITEMS: string[] = [ 'title', 'link', 'value' ];

	editing: ModelWatchParser = this.defaultEditing();

	displaying: { [name: string]: NodeContainer[] } = {};

	preview_url: string = 'https://google.com';

	@ViewChild('frame', { static: true, read: ElementRef })
	public iframeElement!: ElementRef<HTMLIFrameElement>;

	async save() {
		if (this.editing.id == null) {
			this.websocket.send_new_watch_parser(this.editing).then(console.log, console.error);
		} else {
			this.websocket.send_update_watch_parser(this.editing.id, this.editing).then(console.log, console.error);
		}
	}

	async delete() {
		if (this.editing.id == null) {
			this.editing = this.defaultEditing();
		} else {
			this.websocket.send_remove_watch_parser(this.editing.id).then(console.log, console.error);
		}
	}

	async test() {
		// this.websocket.send_test_watcher();
	}

	clone() {
		// this.editing.id = undefined;
		// this.editing.title += ' (Cloned)';
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

		if (this.frameWindow != null && (this.editing.match_opts.items as string).length != 0) {
			let iframe_doc = this.frameWindow.document;

			try {
				let val = iframe_doc.evaluate(this.editing.match_opts.items as string, iframe_doc, null, XPathResult.ANY_TYPE, null);

				let items: Node[] = [];

				let last: Node | null = null;
				while ((last = val.iterateNext()) != null) {
					items.push(last);
				}

				items.forEach(i => (<HTMLElement>i).classList.add(getFrameColor('items')));

				this.displaying.items = items.map(i => new NodeContainer(i));

				this.previewUpdateChildren();

				console.log(`Found: ${items.length}`);
			} catch(e) {
				console.error(e);
			}
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

		if (this.editing.match_opts[child_name] == null || (this.editing.match_opts[child_name] as any).xpath.length == 0) {
			return;
		}

		let search_opts_item = this.editing.match_opts[child_name] as { xpath: string; parse_type: rust.EnumValue; };

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
		} catch(e) {
			console.error(e);
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
		if (this.editing.match_opts[name]) {
			(this.editing.match_opts[name] as any).xpath = xpath;
		} else {
			this.editing.match_opts[name] = {
				xpath: xpath,
				parse_type: 'None'
			};
		}
	}

	setSearchItemParser(name: string, parse_name: string) {
		if (this.editing.match_opts[name]) {
			(this.editing.match_opts[name] as any).parse_type = defaultParserType(parse_name);
		} else {
			this.editing.match_opts[name] = {
				xpath: '',
				parse_type: defaultParserType(parse_name)
			};
		}
	}

	getSearchItemParser(name: string) {
		if (this.editing.match_opts[name]) {
			let parse_type = (this.editing.match_opts[name] as any).parse_type;

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

	setEditing(value: ModelWatchParser) {
		this.editing = value;
	}

	defaultEditing(): ModelWatchParser {
		return {
			title: '',
			description: '',
			match_url: '',
			match_opts: {}
		};
	}
}

function defaultParserType(name: string): rust.EnumNone | rust.EnumObject {
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
		case 'value': return 'editor-border-value';

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

	.editor-border-value {
		border: #AA0 1px solid;
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