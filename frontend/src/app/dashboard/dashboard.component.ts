import { Component } from '@angular/core';
import { BackgroundService } from '../background.service';
import { WebsocketService } from '../websocket.service';

@Component({
	selector: 'app-dashboard',
	templateUrl: './dashboard.component.html',
	styleUrls: ['./dashboard.component.scss']
})

export class DashboardComponent {
	constructor(public background: BackgroundService, public websocket: WebsocketService) {
		websocket.send_get_request_history_group_list(0, 50)
		.then(v => {
			this.group_items = v.groups.map(group => {
				return {
					open: false,
					group,
					items: v.items.filter(v => v.group_id == group.id)
				};
			});
		}, console.error);
	}

	group_items: GroupItem[] = [];


	countErrors(value: ModelRequestHistoryItem[]) {
		return value.filter(v => v.error != null).length
	}

	countSuccesses(value: ModelRequestHistoryItem[]) {
		return value.filter(v => v.error == null).length
	}


	findWatcherOrFeed(watcher_id: Optional<number>, feed_id: Optional<number>) {
		if (watcher_id) {
			return this.background.watching_listeners.find(v => v[0].id == watcher_id)?.[0];
		} else if (feed_id) {
			return this.background.feed_list.find(v => v.id == feed_id);
		} else {
			throw 'Unreachable';
		}
	}

	// TODO: Pipe.
	dateToTimeAgo(value: number) {
		value = Math.floor((Date.now() - value) / 1000);

		let combined = '';

		let days = Math.floor(value / 86_400);
		let hours = Math.floor((value % 86_400) / 3_600);
		let minutes = Math.floor((value % 3_600) / 60);
		let seconds = Math.floor(value % 60);

		if (days != 0) combined += `${days} days `;
		if (hours != 0) combined += `${hours} hours `;
		if (minutes != 0) combined += `${minutes} minutes `;
		if (seconds != 0) combined += `${seconds} seconds`;
		if (combined.length == 0) combined = '0 seconds';

		return combined.trim();
	}
}


interface GroupItem {
	open: boolean;
	group: ModelRequestHistoryGroup;
	items: ModelRequestHistoryItem[];
}