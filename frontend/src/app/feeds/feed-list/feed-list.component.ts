import { Component, OnInit } from '@angular/core';
import { PageEvent } from '@angular/material/paginator';
import { ActivatedRoute, Params, Router } from '@angular/router';
import { FeedItem } from 'src/app/item/feed-item';
import { WebsocketService } from 'src/app/websocket.service';
import { BackgroundService } from '../../background.service';

@Component({
	selector: 'app-feed-list',
	templateUrl: './feed-list.component.html',
	styleUrls: ['./feed-list.component.scss']
})

export class FeedListComponent implements OnInit {
	displayedColumns: string[] = ['from', 'title', 'date_added', 'link'];

	total_item_count: number = 0;
	page_index: number = 0;
	page_size: number = 25;

	viewing_category: number | null = null;
	search_params: string | null = null;


	categories: ModelCategory[] = [];
	category_feeds: ModelFeedCategory[] = [];

	feed_items: FeedItem[] = [];

	constructor(
		public websocket: WebsocketService,
		public background: BackgroundService,
		private route: ActivatedRoute,
		private router: Router
	) {}

	ngOnInit() {
		this.route.queryParams
		.subscribe(params => {
			this.search_params = (params.search == null || params.search.length == 0 ? null : params.search);

			let cat = (params.cat == null ? null : parseInt(params.cat));

			if (cat != null && !isNaN(cat)) {
				this.viewing_category = cat;
				this.refreshFeeds();
			}
		});

		this.background.new_feed_items.subscribe(resp => this.handleListResponse(resp));

		this.refreshFeeds().catch(console.error);
	}

	async refreshFeeds() {
		this.feed_items = [];

		let cats = await this.websocket.send_get_category_list();

		this.categories = cats.categories;
		this.category_feeds = cats.category_feeds;

		let list_resp = await this.websocket.send_get_item_list(
			this.search_params,
			this.viewing_category,
			this.page_index * this.page_size,
			this.page_size
		);

		this.handleListResponse(list_resp);
	}

	handleListResponse(resp: ItemListResponse) {
		if (this.page_index != 0 || this.search_params != null || this.viewing_category != null) {
			return;
		}

		this.total_item_count = resp.total_items;

		resp.items.map(v => new FeedItem(v, resp.notification_ids.includes(v.id!)))
		.forEach(item => {
			if (this.feed_items.findIndex(i => i.id == item.id) == -1) {
				this.feed_items.push(item);
			}
		});

		this.feed_items.sort((a, b) => b.date_added - a.date_added);

		// Remove extras from page.
		this.feed_items.splice(this.page_size, this.feed_items.length);
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

		this.search_params = (value.length == 0 ? null : value);

		this.refreshFeeds();
	}

	paginatorEvent(event: PageEvent) {
		this.page_index = event.pageIndex;
		this.page_size = event.pageSize;

		this.refreshFeeds().catch(console.error);
	}
}