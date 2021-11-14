import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
	name: 'dateGroupSectioning',
	pure: false
})
export class DateGroupSectioning implements PipeTransform {
	transform(value: ModelItem[], args?: any): FeedGrouping[] {
		let groupings: FeedGrouping[] = [];

		let current_section = -1;

		value.forEach(r => {
			let section = get_section_from_date(r.date * 1000);

			if (section != current_section) {
				current_section = section;

				groupings.push({
					title: SECTION_NAMES[section],
					feed_items: []
				});
			}

			groupings[groupings.length - 1].feed_items.push(r);
		});

		return groupings;
	}
}

interface FeedGrouping {
	title: string;
	feed_items: ModelItem[];
}

const SECTION_NAMES = [
	'Today',
	'Yesterday',
	'This Week',
	'This Month',
	'Last Month',
	'This Year',
	'Last Year',
	'Years Ago'
];

function get_section_from_date(timestamp: number): number {
	const NOW = Date.now();
	const DAY = 1000 * 60 * 60 * 24;

	// Today
	if (NOW - timestamp < DAY) return 0;

	// Yesterday
	if (NOW - timestamp < DAY * 2) return 1;

	// This Week
	if (NOW - timestamp < DAY * 7) return 2;

	// This Month
	if (NOW - timestamp < DAY * 30) return 3;

	// Last Month
	if (NOW - timestamp < DAY * 30 * 2) return 4;

	// This Year
	if (NOW - timestamp < DAY * 365) return 5;

	// Last Year
	if (NOW - timestamp < DAY * 365 * 2) return 6;

	return 7;
}