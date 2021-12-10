import { Component, OnInit } from '@angular/core';

import { COMMA, ENTER } from '@angular/cdk/keycodes';

import { BackgroundService } from '../../background.service';
import { MatAutocompleteSelectedEvent } from '@angular/material/autocomplete';
import { WebsocketService } from 'src/app/websocket.service';
import { FeedListener } from 'src/app/item/feed-listener';


@Component({
	selector: 'app-changes-websites',
	templateUrl: './listeners.component.html',
	styleUrls: ['./listeners.component.scss']
})
export class ListenersComponent {
	constructor(public background: BackgroundService, private websocket: WebsocketService) {}

	toggledListener(listener: ModelWatcher, value: boolean) {
		if (listener.id != null) {
			listener.enabled = value;
			this.websocket.send_edit_watcher(listener.id, { enabled: value });
		}
	}

	updateInterval(listener: ModelWatcher, interval: number) {
		if (listener.id != null && interval != 0) {
			listener.sec_interval = interval;
			this.websocket.send_edit_watcher(listener.id, { sec_interval: interval });
		}
	}

	addListener(value: string, id?: string) {
		if (id != null) {
			this.websocket.send_create_watcher(value, parseInt(id))
			.then(
				resp => {
					console.log(resp);
					// this.background.watching_listeners.push([new FeedListener(resp.listener), ]);
				},
				console.error
			);
		}
	}

	deleteListener(id: number | undefined, rem_stored: boolean) {
		console.log(id, rem_stored);

		if (id != null) {
			this.websocket.send_remove_watcher(id, rem_stored)
			.then(
				resp => {
					console.log(resp);
					this.background.watching_listeners.splice(this.background.watching_listeners.findIndex(v => v[0].id == id), 1);
				},
				console.error
			);
		}
	}

	getParserById(id: number) {
		return this.background.watch_parser.find(v => v.id == id);
	}
}