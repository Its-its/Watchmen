import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Params, Router } from '@angular/router';
import { BackgroundService } from '../../background.service';

@Component({
	selector: 'app-feed-list',
	templateUrl: './feed-list.component.html',
	styleUrls: ['./feed-list.component.scss']
})

export class FeedListComponent implements OnInit {
	displayedColumns: string[] = ['from', 'title', 'date_added', 'link'];

	constructor(
		public background: BackgroundService,
		private route: ActivatedRoute,
		private router: Router
	) {}

	ngOnInit() {
		this.route.queryParams
		.subscribe(params => {
			this.background.search_params = (params.search == null || params.search.length == 0 ? null : params.search);

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

	searchParams(value: string) {
		this.router.navigate(
			[],
			{
				relativeTo: this.route,
				queryParams: { search: value },
				queryParamsHandling: 'merge'
			}
		);

		this.background.search_params = (value.length == 0 ? null : value);

		this.background.reset_feeds();
	}
}