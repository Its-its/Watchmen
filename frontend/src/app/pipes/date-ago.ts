import { S } from '@angular/cdk/keycodes';
import { Pipe, PipeTransform } from '@angular/core';

// Arguments:
// 	- m: Date contains Milliseconds
// 	- a: Add "ago" to string

const INCLUDES_MILLISECONDS = 'm';
const ADD_AGO_TO_END = 'a';
const VALUE_IS_SECONDS = 's';
const MULTIPLE_INTERVALS = 'c';

@Pipe({
	name: 'dateAgo',
	pure: true
})
export class DateAgoPipe implements PipeTransform {
	transform(value: any, args?: any): any {
		if (value) {
			let seconds;

			// If value is already seconds, just set it.
			if (args.includes(VALUE_IS_SECONDS)) {
				seconds = value;
			} else {
				// Value contains Milliseconds, remove it.
				if (args != null && args.includes(INCLUDES_MILLISECONDS)) {
					value = Math.floor(value / 1000);
				}

				seconds = Math.floor(Date.now() / 1000) - value;
			}

			if (seconds == 0) {
				return '0 seconds' + (args != null && args.includes(ADD_AGO_TO_END) ? ' ago' : '');
			}

			let comp = '';

			for (const name in INTERVALS) {
				let counter = Math.floor(seconds / INTERVALS[name]);

				if (counter > 0) {
					// Add interval to string.
					comp += `${counter} ${name}`;

					// Remove interval seconds from value.
					seconds %= INTERVALS[name];

					// singular or plural
					if (counter !== 1) {
						comp += 's';
					}

					// Add "ago" to end of string
					if (args != null && args.includes(ADD_AGO_TO_END)) comp += ' ago';

					if (args != null && args.includes(MULTIPLE_INTERVALS)) {
						comp += ', ';
					} else {
						// Only using 1 interval. Return it.
						return comp;
					}
				}
			}

			// Remove ", " and return (used for multiple intervals.)
			return comp.slice(0, comp.length - 2);
		} else {
			return value;
		}
	}
}

const INTERVALS: { [value: string]: number } = {
	'year': 31_536_000,
	'month': 2_592_000,
	'week': 604_800,
	'day': 86_400,
	'hour': 3_600,
	'minute': 60,
	'second': 1
};