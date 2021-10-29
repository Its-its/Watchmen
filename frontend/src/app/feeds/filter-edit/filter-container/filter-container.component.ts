import { Component, Input, OnInit } from '@angular/core';

@Component({
	selector: 'filter-container',
	templateUrl: './filter-container.component.html',
	styleUrls: ['./filter-container.component.scss']
})

export class FilterContainerComponent implements OnInit {
	ngOnInit(): void {
		console.log(this.filterName, this.container);
	}

	@Input() isStartingFilter: boolean = false;
	@Input() filterName: string = '';
	@Input() container: rust.EnumValue = {};


	as_array_objects(): rust.EnumObject[] {
		return this.container as rust.EnumObject[];
	}

	as_array_values(): rust.Values[] {
		return this.container as rust.Values[];
	}

	get_array_value<V = rust.Values>(index: number): V {
		return this.as_array_values()[index] as any;
	}

	getFilterByString() {
		switch (this.filterName) {
			case FilterBy.And.toString(): return FilterBy.And;
			case FilterBy.Or.toString(): return FilterBy.Or;
			case FilterBy.Contains.toString(): return FilterBy.Contains;
			case FilterBy.Regex.toString(): return FilterBy.Regex;
			case FilterBy.StartsWith.toString(): return FilterBy.StartsWith;
			case FilterBy.EndsWith.toString(): return FilterBy.EndsWith;
			default: throw 'Unreachable';
		}
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