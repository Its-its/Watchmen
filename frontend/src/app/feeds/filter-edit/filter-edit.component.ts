import { Component } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';
import { WebsocketService } from 'src/app/websocket.service';

@Component({
	selector: 'app-filter-edit',
	templateUrl: './filter-edit.component.html',
	styleUrls: ['./filter-edit.component.scss']
})

export class FilterEditComponent {
	editingFilter: Nullable<FilterModel> = null;

	constructor(public background: BackgroundService, private websocket: WebsocketService) { }

	setEditFilter(filter: FilterModel) {
		this.editingFilter = filter;
	}

	onTitleChange(event: Event) {
		this.editingFilter && (this.editingFilter.title = (event.target as HTMLInputElement).value);
	}

	onButtonUpdate() {
		if (this.editingFilter != null && this.editingFilter.filter != null) {
			if (this.editingFilter.id == null) {
				this.websocket.send_new_filter(
					this.editingFilter.title,
					this.editingFilter.filter
				).then(v => {
					console.log(v);
					setTimeout(() => window.location.reload(), 1000);
				}, console.error);
			} else {
				this.websocket.send_update_filter(
					this.editingFilter.id,
					this.editingFilter.title,
					this.editingFilter.filter
				).then(console.log, console.error);
			}
		}
	}

	onButtonClone() {
		if (this.editingFilter != null) {
			this.editingFilter = JSON.parse(JSON.stringify(this.editingFilter));

			this.editingFilter!.id = undefined;
			this.editingFilter!.title += ' (cloned)';
		}
	}

	onButtonDelete() {
		if (this.editingFilter != null) {
			// Send delete command
			if (this.editingFilter.id != null) {
				this.websocket.send_remove_filter(this.editingFilter.id)
				.then(console.log, console.error);

				let index = this.background.filter_list.findIndex(v => v.filter.id == this.editingFilter!.id);

				this.background.filter_list.splice(index, 1);
			}

			this.editingFilter = null;
		}
	}


	onCreateFilter(filter: string) {
		if (this.editingFilter) {
			this.editingFilter.filter = {
				[filter]: defaultFilterOptions(filter)
			};
		} else {
			this.editingFilter = {
				id: undefined,
				title: 'New Filter',
				filter: {
					[filter]: defaultFilterOptions(filter)
				}
			};
		}
	}

	onDeleteFilter() {
		this.editingFilter && (this.editingFilter.filter = undefined);
	}
}

function defaultFilterOptions(filter: string): any {
	switch (filter) {
		case 'And': return [];
		case 'Or': return [];
		case 'Contains': return [ '', false ];
		case 'Regex': return [ '', {
			dot_matches_new_line: false,
			ignore_whitespace: false,
			case_insensitive: true,
			multi_line: false,
			swap_greed: false,
			unicode: true,
			octal: false
		}];
		case 'StartsWith': return [ '', false ];
		case 'EndsWith': return [ '', false ];
	}
}
