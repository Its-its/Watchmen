import { Component } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';

@Component({
	selector: 'app-filter-edit',
	templateUrl: './filter-edit.component.html',
	styleUrls: ['./filter-edit.component.scss']
})

export class FilterEditComponent {
	editingFilter: Nullable<FilterModel> = null;

	constructor(public background: BackgroundService) { }

	setEditFilter(filter: FilterModel) {
		this.editingFilter = filter;
	}
}