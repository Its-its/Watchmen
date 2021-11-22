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
}


interface GroupItem {
	open: boolean;
	group: ModelRequestHistoryGroup;
	items: ModelRequestHistoryItem[];
}