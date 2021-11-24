import { Component, Input } from '@angular/core';

@Component({
	selector: 'app-toolbar',
	templateUrl: './toolbar.component.html',
	styleUrls: ['./toolbar.component.scss']
})

export class ToolbarComponent {
	items = [
		["Dashboard", "/dashboard"],
		null,
		["Feed List", "/feeds"],
		["Feed Watching", "/feeds/watching"],
		["Feed Editor", "/feeds/editor"],
		["Feed Filter", "/feeds/filter"],
		null,
		["Change List", "/changes"],
		["Change Editor", "/changes/editor"]
	];

	constructor() {}
}