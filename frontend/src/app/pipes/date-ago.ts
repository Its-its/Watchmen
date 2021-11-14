import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
	name: 'dateAgo',
	pure: true
})
export class DateAgoPipe implements PipeTransform {
	transform(value: any, args?: any): any {
		if (value) {
			const SECONDS = Math.floor(Date.now() / 1000) - value;

			if (SECONDS < 29) { // less than 30 seconds ago will show as 'Just now'
				return 'Just now';
			}

			let counter;

			for (const i in INTERVALS) {
				counter = Math.floor(SECONDS / INTERVALS[i]);

				if (counter > 0) {
					if (counter === 1) {
						return counter + ' ' + i + ' ago'; // singular (1 day ago)
					} else {
						return counter + ' ' + i + 's ago'; // plural (2 days ago)
					}
				}
			}
		}

		return value;
	}
}

const INTERVALS: { [value: string]: number } = {
	'year': 31536000,
	'month': 2592000,
	'week': 604800,
	'day': 86400,
	'hour': 3600,
	'minute': 60,
	'second': 1
};