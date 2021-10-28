import { Component, OnInit } from '@angular/core';

import { COMMA, ENTER } from '@angular/cdk/keycodes';

import { BackgroundService } from '../../background.service';
import { MatAutocompleteSelectedEvent } from '@angular/material/autocomplete';
import { WebsocketService } from 'src/app/websocket.service';


export interface Fruit {
	name: string;
}


@Component({
	selector: 'app-websites',
	templateUrl: './websites.component.html',
	styleUrls: ['./websites.component.scss']
})

export class WebsitesComponent implements OnInit {
	constructor(public background: BackgroundService, private websocket: WebsocketService) {}

	addOnBlur = true;

	readonly separatorKeysCodes = [ENTER, COMMA] as const;

	remove(feed_id: number, filter_group: FilterGroupListener): void {
		const index = filter_group.feeds.indexOf(feed_id);

		if (index != -1) {
			filter_group.feeds.splice(index, 1);
			this.websocket.send_remove_feed_filter(feed_id, filter_group.filter.id).then(console.log, console.error);
		}
	}

	selected(feed_id: number, event: MatAutocompleteSelectedEvent): void {
		const filter_id = parseInt(event.option._getHostElement().getAttribute('data-filter-id')!);

		let filter = this.background.filter_list.find(v => v.filter.id == filter_id);

		if (filter != null) {
			filter.feeds.push(feed_id);
			this.websocket.send_new_feed_filter(feed_id, filter.filter.id).catch(console.error);
		}
	}

	toggedWebsite(feed_id: number, value: boolean) {
		let feed = this.background.feed_list.find(v => v.id == feed_id);

		if (feed != null) {
			feed.enabled = value;
			this.websocket.send_edit_listener(feed_id, { enabled: value });
		}
	}

	_filter(value: string): string[] {
		const filterValue = value.toLowerCase();
		return this.background.filter_list.map(v => v.filter.title).filter(item => item.toLowerCase().includes(filterValue));
	}

	ngOnInit(): void {
		//
	}
}