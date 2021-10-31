import { Component, ElementRef, EventEmitter, Input, OnChanges, OnInit, Output } from '@angular/core';

@Component({
	selector: 'filter-container',
	templateUrl: './filter-container.component.html',
	styleUrls: ['./filter-container.component.scss']
})

export class FilterContainerComponent implements OnChanges {
	@Input() isStartingFilter: boolean = false;
	@Input() filterName: string = '';
	@Input() container: rust.EnumValue = {};

	@Output() deleteButtonClicked = new EventEmitter<undefined>();

	constructor(private elementRef: ElementRef) {}

	ngOnChanges(): void {
		console.log(this.filterName, this.container);
	}


	as_array_objects(): rust.EnumObject[] {
		return this.container as rust.EnumObject[];
	}

	as_array_values(): rust.Values[] {
		return this.container as rust.Values[];
	}

	get_array_value<V = rust.Values>(index: number): V {
		return this.as_array_values()[index] as any;
	}


	// Events

	onDeleteChild(index: number) {
		this.as_array_objects().splice(index, 1);
	}

	onAddFilter(filter: string) {
		this.as_array_objects().push({
			[filter]: defaultFilterOptions(getFilterByString(filter))
		});
	}

	onClickDeleteButton() {
		this.deleteButtonClicked.emit();
	}
}

export enum FilterBy {
	Regex,
	Contains,
	StartsWith,
	EndsWith,
	And,
	Or
}


function getFilterByString(filterName: string) {
	switch (filterName) {
		case "And": return FilterBy.And;
		case "Or": return FilterBy.Or;
		case "Contains": return FilterBy.Contains;
		case "Regex": return FilterBy.Regex;
		case "StartsWith": return FilterBy.StartsWith;
		case "EndsWith": return FilterBy.EndsWith;
		default: throw 'Unreachable';
	}
}

function defaultFilterOptions(filter: FilterBy): any {
	switch (filter) {
		case FilterBy.And: return [];
		case FilterBy.Or: return [];
		case FilterBy.Contains: return [ '', false ];
		case FilterBy.Regex: return {
			dot_matches_new_line: false,
			ignore_whitespace: false,
			case_insensitive: true,
			multi_line: false,
			swap_greed: false,
			unicode: true,
			octal: false
		};
		case FilterBy.StartsWith: return [ '', false ];
		case FilterBy.EndsWith: return [ '', false ];
	}
}

// "filter": {
// 	"And": [
// 		{
// 			"Contains": [
// 				"ssd",
// 				false
// 			]
// 		},
// 		{
// 			"Contains": [
// 				"m.2",
// 				false
// 			]
// 		}
// 	]
// }

// {
// 	"And": [
// 	  {
// 		"Contains": [
// 		  "hdd",
// 		  false
// 		]
// 	  },
// 	  {
// 		"Or": [
// 		  {
// 			"Contains": [
// 			  "14tb",
// 			  false
// 			]
// 		  },
// 		  {
// 			"Contains": [
// 			  "14 tb",
// 			  false
// 			]
// 		  },
// 		  {
// 			"Contains": [
// 			  "12tb",
// 			  false
// 			]
// 		  },
// 		  {
// 			"Contains": [
// 			  "12 tb",
// 			  false
// 			]
// 		  },
// 		  {
// 			"Contains": [
// 			  "16tb",
// 			  false
// 			]
// 		  },
// 		  {
// 			"Contains": [
// 			  "16 tb",
// 			  false
// 			]
// 		  }
// 		]
// 	  }
// 	]
// }