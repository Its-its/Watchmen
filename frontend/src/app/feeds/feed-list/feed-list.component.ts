import { Component, OnInit } from '@angular/core';
import { BackgroundService } from '../../background.service';
import { WebsocketService } from '../../websocket.service';

interface Feed {
	id?: number;

	url: string;
	title: string;
	description: string;

	generator: string;

	global_show: boolean;
	ignore_if_not_new: boolean;

	remove_after: number;
	sec_interval: number;

	date_added: number;
	last_called: number;
}

interface FeedItem {
	id?: number;

	guid: string;
	title: string;
	author: string;
	content: string;
	link: string;
	date: number;
	hash: string;

	date_added: number;
	is_read: boolean;
	is_starred: boolean;
	is_removed: boolean;
	tags: string;
	feed_id: number;
}

interface FeedGrouping {
	title: string;
	feed_items: FeedItem[];
}


@Component({
	selector: 'app-feed-list',
	templateUrl: './feed-list.component.html',
	styleUrls: ['./feed-list.component.scss']
})

export class FeedListComponent implements OnInit {
	displayedColumns: string[] = ['from', 'title', 'date_added', 'link'];

	constructor(public background: BackgroundService) {}

	ngOnInit(): void {
		//
	}

	getLocaleDateString(date: number): string {
		return new Date(date * 1000).toLocaleString();
	}

	sorted_feeds(): FeedGrouping[] {
		let groupings: FeedGrouping[] = [];

		let current_section = -1;

		this.background.feed_items.forEach(r => {
			let section = get_section_from_date(r.date * 1000);

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

const SECTION_NAMES = [
	'Today',
	'Yesterday',
	'This Week',
	'This Month',
	'Last Month',
	'This Year',
	'Last Year'
];

function get_section_from_date(timestamp: number): number {
	const now = Date.now();
	const day = 1000 * 60 * 60 * 24;

	// Last Year
	if (timestamp < now - (day * 365 * 2)) return 6;

	// This Year
	if (timestamp < now - (day * 365 * 2)) return 5;

	// Last Month
	if (timestamp < now - (day * 30 * 2)) return 4;

	// This Month
	if (timestamp < now - (day * 30)) return 3;

	// This Week
	if (timestamp < now - (day * 7)) return 2;

	// Yesterday
	if (timestamp < now - day) return 1;

	return 0;
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