import { Component, OnInit } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';
import { WebsocketService } from 'src/app/websocket.service';

@Component({
  selector: 'app-list',
  templateUrl: './list.component.html',
  styleUrls: ['./list.component.scss']
})

export class ListComponent {
	displayedColumns: string[] = ['active', 'title', 'message', 'activity', 'link'];



	constructor(public background: BackgroundService, public websocket: WebsocketService) {}

	getLocaleDateString(date: number): string {
		return new Date(date * 1000).toLocaleString();
	}

	toggleEnabled(id: number, value: boolean) {
		let listener = this.background.watching_listeners.find(v => v[0].id == id);

		if (listener != null) {
			listener[0].enabled = value;
			this.websocket.send_edit_watcher(listener[0].id!, { enabled: listener[0].enabled })
				.then(console.log, console.error);
		}
	}

	sorted_watching(): FeedGrouping[] {
		console.log('asdf');
		let current_section = -1;

		let groupings: FeedGrouping[] = [];

		let rows = this.background.watching_listeners;
		rows.sort((a, b) => b[1].date_added - a[1].date_added);

		rows.forEach(r => {
			let section = get_section_from_date(r[1].date_added * 1000);

			console.log(new Date(r[1].date_added * 1000).toLocaleTimeString() + ' - ' + section);

			if (section != current_section) {
				current_section = section;

				groupings.push({
					title: SECTION_NAMES[section],
					feed_items: []
				});
			}

			groupings[groupings.length - 1].feed_items.push(r);
		});

		return groupings;
	}

	public parse_timestamp(date: number): string {
		date = date * 1000;

		if (date + (1000 * 60 * 60 * 24) > Date.now()) {
			return elapsed_to_time_ago(Date.now() - date);
		} else {
			return new Date(date).toLocaleString()
		}
	}
}

interface FeedGrouping {
	title: string;
	feed_items: [ModelWatcher, ModelWatchHistory][];
}


const SECTION_NAMES = [
	'Today',
	'Yesterday',
	'This Week',
	'This Month',
	'Last Month',
	'This Year',
	'Last Year',
	'Years Ago'
];
// TODO: Place into Util file.
function get_section_from_date(timestamp: number): number {
	const NOW = Date.now();
	const DAY = 1000 * 60 * 60 * 24;

	// Today
	if (NOW - timestamp < DAY) return 0;

	// Yesterday
	if (NOW - timestamp < DAY * 2) return 1;

	// This Week
	if (NOW - timestamp < DAY * 7) return 2;

	// This Month
	if (NOW - timestamp < DAY * 30) return 3;

	// Last Month
	if (NOW - timestamp < DAY * 30 * 2) return 4;

	// This Year
	if (NOW - timestamp < DAY * 365) return 5;

	// Last Year
	if (NOW - timestamp < DAY * 365 * 2) return 6;

	return 7;
}

function elapsed_to_time_ago(elapsed: number): string {
	let msPerMinute = 60 * 1000;
	let msPerHour = msPerMinute * 60;

	if (elapsed < msPerMinute) {
		return Math.floor(elapsed/1000) + 's ago';
	}

	if (elapsed < msPerHour) {
		return Math.floor(elapsed/msPerMinute) + 'm ago';
	}

	return `${Math.floor(elapsed/msPerHour)}h, ${Math.floor(elapsed/msPerMinute) % 60}m ago`;
}