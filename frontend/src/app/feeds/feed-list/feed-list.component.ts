import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { WebsocketService } from 'src/app/websocket.service';
import { BackgroundService } from '../../background.service';

@Component({
	selector: 'app-feed-list',
	templateUrl: './feed-list.component.html',
	styleUrls: ['./feed-list.component.scss']
})

export class FeedListComponent implements OnInit {
	displayedColumns: string[] = ['from', 'title', 'date_added', 'link'];

	constructor(public background: BackgroundService, private route: ActivatedRoute) {}

	ngOnInit() {
		this.route.queryParams
		.subscribe(params => {
			let cat = (params.cat == null ? null : parseInt(params.cat));

			if (cat != null && !isNaN(cat)) {
				this.background.viewing_category = cat;
				this.background.reset_feeds();
			}
		});
	}

	getLocaleDateString(date: number): string {
		return new Date(date * 1000).toLocaleString();
	}
}