import { Component, ElementRef, EventEmitter, Input, OnChanges, OnInit, Output } from '@angular/core';
import { MatCheckboxChange } from '@angular/material/checkbox';
import { MatSelectionListChange } from '@angular/material/list';

@Component({
	selector: 'filter-container',
	templateUrl: './filter-container.component.html',
	styleUrls: ['./filter-container.component.scss']
})

export class FilterContainerComponent {
	@Input() filterName: string = '';
	@Input() container: rust.EnumValue = {};

	@Output() deleteButtonClicked = new EventEmitter<undefined>();

	constructor() {}

	as_array_objects(): rust.EnumObject[] {
		return this.container as rust.EnumObject[];
	}

	as_array_values(): rust.Values[] {
		return this.container as rust.Values[];
	}

	get_array_value<V = rust.Values>(index: number): V {
		return this.as_array_values()[index] as any;
	}

	get_regex_object(): [string, { [name: string]: boolean }] {
		return this.container as any;
	}


	// Events

	onChange(index: number, event: Event | MatCheckboxChange | MatSelectionListChange) {
		if (event instanceof Event) {
			this.as_array_values()[index] = (event.target as any).value;
		} else if (event instanceof MatCheckboxChange) {
			this.as_array_values()[index] = event.checked;
		} else if (event instanceof MatSelectionListChange) {
			event.options.forEach(v => this.get_regex_object()[1][v.value] = v.selected);
		}
	}

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
		case FilterBy.Regex: return [ '', {
			dot_matches_new_line: false,
			ignore_whitespace: false,
			case_insensitive: true,
			multi_line: false,
			swap_greed: false,
			unicode: true,
			octal: false
		}];;
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