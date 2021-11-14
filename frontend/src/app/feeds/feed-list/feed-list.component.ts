import { Component } from '@angular/core';
import { BackgroundService } from '../../background.service';

@Component({
	selector: 'app-feed-list',
	templateUrl: './feed-list.component.html',
	styleUrls: ['./feed-list.component.scss']
})

export class FeedListComponent {
	displayedColumns: string[] = ['from', 'title', 'date_added', 'link'];

	constructor(public background: BackgroundService) {}

	getLocaleDateString(date: number): string {
		return new Date(date * 1000).toLocaleString();
	}
}