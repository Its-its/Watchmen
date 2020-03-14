type ParseFunc = (_: string, date: Date) => Nullable<number>;


abstract class FormatParser {
	format: Format;

	constructor(format: Format) {
		this.format = format;
	}

	parse(_: string, _1: Date): Nullable<number> { return null; };
}


class LiteralFormat extends FormatParser {
	char: string;

	constructor(char: string) {
		super(Format.Literal);
		this.char = char;
	}

	parse(value: string): Nullable<number> {
		return value.startsWith(this.char) ? this.char.length : null;
	}
}

function registerLiteral(char: string): FormatParser {
	return new LiteralFormat(char);
}

function createFormat(format: Format, parse: ParseFunc): () => FormatParser {
	class CustomFormat extends FormatParser {
		constructor() {
			super(format);
		}

		parse(value: string, date: Date): Nullable<number> {
			return parse(value, date);
		}
	}

	return () => new CustomFormat();
}


enum Format {
	// Date
	FullYearGregorianPadded, // 2001
	ProlepticGregorianYearDivided, // 20
	ProlepticGregorianYearModulo, // 01

	MonthNumberPadded,
	AbbrMonthName,
	FullMonthName,

	DayNumberPadded,
	DayNumber,

	AbbrWeekdayName,
	FullWeekdayName,
	WeekdayNumberSunday,
	WeekdayNumberMonday,

	WeekNumberSundayPadded,
	WeekNumberMondayPadded,

	FullYearISO_8601,
	ProlepticYearISO_8601,
	WeekNumberISO_8601,

	DayOfYearPadded,

	MonthDayYear,
	YearMonthDayISO_8601,
	DayMonthYear,

	// Time
	HourPadded_24,
	Hour_24,
	HourPadded_12,
	Hour_12,

	AM_PM_LOWER,
	AM_PM_UPPER,

	MinutePadded,
	SecondPadded,
	FractionalSeconds,
	FractionalSecondsPrecision,
	FractionalSeconds_3_Dot,
	FractionalSeconds_6_Dot,
	FractionalSeconds_9_Dot,
	FractionalSeconds_3,
	FractionalSeconds_6,
	FractionalSeconds_9,

	HourMinute,
	HourMinuteSecond_24,
	HourMinuteSecond_12,


	// Time Zone
	LocalTimeZoneName,
	OffsetFromLocaltimeFull,
	OffsetFromLocaltimeColon,
	OffsetFromLocaltimeHour,

	// Date & Time
	DateAndTime,
	DateAndTime_ISO_8601_RFC_3339,

	UnixTimestamp,

	// Special
	LiteralTab,
	LiteralNewLine,
	LiteralPercentSign,

	// Padding override for numeric specifiers.
	SupressPadding,
	IncludeSpacePadding,
	IncludeZeroPadding,

	Literal
}

const MONTH_NAMES_FULL = [
	'January', 'Februrary', 'March', 'April', 'May', 'June',
	'July', 'August', 'September', 'October', 'November', 'December'
];

const MONTH_NAMES_ABBR = [
	'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
	'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
];

const FORMATS: { [type: number]: () => FormatParser } = {
	[Format.FullYearGregorianPadded]: createFormat(Format.FullYearGregorianPadded, value => null),
	[Format.ProlepticGregorianYearDivided]: createFormat(Format.ProlepticGregorianYearDivided, value => null),
	[Format.ProlepticGregorianYearModulo]: createFormat(Format.ProlepticGregorianYearModulo, value => null),
	[Format.MonthNumberPadded]: createFormat(Format.MonthNumberPadded, value => null),
	[Format.AbbrMonthName]: createFormat(Format.AbbrMonthName, (value, date) => {
		let found = MONTH_NAMES_ABBR.map(i => i.toLowerCase()).indexOf(value.slice(0, 3).toLowerCase());

		if (found == -1) return null;

		date.setMonth(found);

		return 3;
	}),
	[Format.FullMonthName]: createFormat(Format.FullMonthName, (value, date) => {
		let val_lower = value.toLowerCase();
		let found = MONTH_NAMES_FULL.findIndex(i => val_lower.startsWith(i.toLowerCase()));

		if (found == -1) return FORMATS[Format.AbbrMonthName]().parse(value, date);

		date.setMonth(found);

		return MONTH_NAMES_FULL[found].length;
	}),

	[Format.DayNumberPadded]: createFormat(Format.DayNumberPadded, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (!isNaN(numb)) {
			date.setDate(numb);
			return 2;
		}

		return null;
	}),
	[Format.DayNumber]: createFormat(Format.DayNumber, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (isNaN(numb)) {
			numb = parseInt(value.slice(0, 1));
			if (isNaN(numb)) return null;
		}

		date.setDate(numb);
		return numb > 9 ? 2 : 1;
	}),

	[Format.AbbrWeekdayName]: createFormat(Format.AbbrWeekdayName, value => null),
	[Format.FullWeekdayName]: createFormat(Format.FullWeekdayName, value => null),
	[Format.WeekdayNumberSunday]: createFormat(Format.WeekdayNumberSunday, value => null),
	[Format.WeekdayNumberMonday]: createFormat(Format.WeekdayNumberMonday, value => null),

	[Format.WeekNumberSundayPadded]: createFormat(Format.WeekNumberSundayPadded, value => null),
	[Format.WeekNumberMondayPadded]: createFormat(Format.WeekNumberMondayPadded, value => null),

	[Format.FullYearISO_8601]: createFormat(Format.FullYearISO_8601, value => null),
	[Format.ProlepticYearISO_8601]: createFormat(Format.ProlepticYearISO_8601, value => null),
	[Format.WeekNumberISO_8601]: createFormat(Format.WeekNumberISO_8601, value => null),

	[Format.DayOfYearPadded]: createFormat(Format.DayOfYearPadded, value => null),

	[Format.MonthDayYear]: createFormat(Format.MonthDayYear, value => null),
	[Format.YearMonthDayISO_8601]: createFormat(Format.YearMonthDayISO_8601, value => null),
	[Format.DayMonthYear]: createFormat(Format.DayMonthYear, value => null),

	// Time
	[Format.HourPadded_24]: createFormat(Format.HourPadded_24, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (isNaN(numb)) return null;

		date.setHours(numb);

		return 2;
	}),
	[Format.Hour_24]: createFormat(Format.Hour_24, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (isNaN(numb)) {
			numb = parseInt(value.slice(0, 1));
			if (isNaN(numb)) return null;
		}

		date.setHours(numb);

		return numb > 9 ? 2 : 1;
	}),
	[Format.HourPadded_12]: createFormat(Format.HourPadded_12, value => null),
	[Format.Hour_12]: createFormat(Format.Hour_12, value => null),

	[Format.AM_PM_LOWER]: createFormat(Format.AM_PM_LOWER, value => null),
	[Format.AM_PM_UPPER]: createFormat(Format.AM_PM_UPPER, value => null),

	[Format.MinutePadded]: createFormat(Format.MinutePadded, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (isNaN(numb)) return null;

		date.setMinutes(numb);

		return 2;
	}),
	[Format.SecondPadded]: createFormat(Format.SecondPadded, (value, date) => {
		let numb = parseInt(value.slice(0, 2));

		if (isNaN(numb)) return null;

		date.setSeconds(numb);

		return 2;
	}),
	[Format.FractionalSeconds]: createFormat(Format.FractionalSeconds, value => null),
	[Format.FractionalSecondsPrecision]: createFormat(Format.FractionalSecondsPrecision, value => null),
	[Format.FractionalSeconds_3_Dot]: createFormat(Format.FractionalSeconds_3_Dot, value => null),
	[Format.FractionalSeconds_6_Dot]: createFormat(Format.FractionalSeconds_6_Dot, value => null),
	[Format.FractionalSeconds_9_Dot]: createFormat(Format.FractionalSeconds_9_Dot, value => null),
	[Format.FractionalSeconds_3]: createFormat(Format.FractionalSeconds_3, value => null),
	[Format.FractionalSeconds_6]: createFormat(Format.FractionalSeconds_6, value => null),
	[Format.FractionalSeconds_9]: createFormat(Format.FractionalSeconds_9, value => null),

	[Format.HourMinute]: createFormat(Format.HourMinute, (value, date) => {
		let sliced = value.slice(0, 5);
		let split = sliced.split(':');

		if (split.length != 2) return null;

		if (FORMATS[Format.HourPadded_24]().parse(split[0], date) == null) return null;
		if (FORMATS[Format.MinutePadded]().parse(split[1], date) == null) return null;

		return sliced.length;
	}),
	[Format.HourMinuteSecond_24]: createFormat(Format.HourMinuteSecond_24, value => null),
	[Format.HourMinuteSecond_12]: createFormat(Format.HourMinuteSecond_12, value => null),

	// Time Zone
	[Format.LocalTimeZoneName]: createFormat(Format.LocalTimeZoneName, (value, date) => {
		let index = value.indexOf(' ');

		if (index == -1) index = value.length;

		try {
			let correctDate = new Date(date.toUTCString().replace('GMT', value.slice(0, index))).getTime();
			date.setTime(correctDate);

			return index;
		} catch (e) {
			console.error(e);
		}

		return null;
	}),
	[Format.OffsetFromLocaltimeFull]: createFormat(Format.OffsetFromLocaltimeFull, value => null),
	[Format.OffsetFromLocaltimeColon]: createFormat(Format.OffsetFromLocaltimeColon, value => null),
	[Format.OffsetFromLocaltimeHour]: createFormat(Format.OffsetFromLocaltimeHour, value => null),

	// Date & Time
	[Format.DateAndTime]: createFormat(Format.DateAndTime, value => null),
	[Format.DateAndTime_ISO_8601_RFC_3339]: createFormat(Format.DateAndTime_ISO_8601_RFC_3339, value => null),

	[Format.UnixTimestamp]: createFormat(Format.UnixTimestamp, value => null),

	// Special
	[Format.LiteralTab]: createFormat(Format.LiteralTab, value => null),
	[Format.LiteralNewLine]: createFormat(Format.LiteralNewLine, value => null),
	[Format.LiteralPercentSign]: createFormat(Format.LiteralPercentSign, value => null),

	// Padding override for numeric specifiers.
	[Format.SupressPadding]: createFormat(Format.SupressPadding, value => null),
	[Format.IncludeSpacePadding]: createFormat(Format.IncludeSpacePadding, value => null),
	[Format.IncludeZeroPadding]: createFormat(Format.IncludeZeroPadding, value => null)
};


function parseFromString(value: string, format: string): Nullable<Date> {
	let parser = new Parser(format);

	while (parser.hasNextChar()) {
		parser.next();
	}

	return parser.parse(value);
}


class Parser {
	format: string;
	pos: number;

	parsed: FormatParser[];

	constructor(format: string) {
		this.pos = 0;
		this.format = format;
		this.parsed = [];
	}

	parse(value: string): Nullable<Date> {
		let date = new Date();
		date.setSeconds(0);
		date.setMinutes(0);
		date.setMilliseconds(0);
		date.setHours(0);

		let pos = 0;

		for (let i = 0; i < this.parsed.length; i++) {
			const format = this.parsed[i];

			let nextStep = format.parse(value.slice(pos), date);

			if (nextStep == null) {
				console.error('Failed to Parse at: ' + value.slice(pos));
				return null;
			}

			pos += nextStep;
		}

		return date;
	}

	next() {
		let char = this.nextChar();

		if (char == null) return;

		switch(char) {
			case '%':
				let loc = this.getSpec(this.nextChar());
				if (loc == null) {
					return console.error('Unable to parse char @ ' + (this.pos - 1));
				} else {
					this.parsed.push(FORMATS[loc]());
				}
				break;

			default:
				// console.log(`Unknown Char: '${char}'`);
				this.parsed.push(registerLiteral(char));
				break;
		}
	}

	getSpec(nextChar: Nullable<string>): Nullable<Format> {
		switch(nextChar) {
			// Date Specifiers
			case 'Y': return Format.FullYearGregorianPadded;
			case 'C': return Format.ProlepticGregorianYearDivided;
			case 'y': return Format.ProlepticGregorianYearModulo;

			case 'm': return Format.MonthNumberPadded;
			case 'b':
			case 'h': return Format.AbbrMonthName;
			case 'B': return Format.FullMonthName;

			case 'd': return Format.DayNumberPadded;
			case 'e': return Format.DayNumber;

			case 'a': return Format.AbbrWeekdayName;
			case 'A': return Format.FullWeekdayName;
			case 'w': return Format.WeekdayNumberSunday;
			case 'u': return Format.WeekdayNumberMonday;

			case 'U': return Format.WeekNumberSundayPadded;
			case 'W': return Format.WeekNumberMondayPadded;

			case 'G': return Format.FullYearISO_8601;
			case 'g': return Format.ProlepticYearISO_8601;
			case 'V': return Format.WeekNumberISO_8601;

			case 'j': return Format.DayOfYearPadded;

			case 'x':
			case 'D': return Format.MonthDayYear;
			case 'F': return Format.YearMonthDayISO_8601;
			case 'v': return Format.DayMonthYear;

			// Time Specifiers
			case 'H': return Format.HourPadded_24;
			case 'k': return Format.Hour_24;
			case 'I': return Format.HourPadded_12;
			case 'l': return Format.Hour_12;

			// AM / PM
			case 'P': return Format.AM_PM_LOWER;
			case 'p': return Format.AM_PM_UPPER;

			// Minute Second
			case 'M': return Format.MinutePadded;
			case 'S': return Format.SecondPadded;
			case 'f': return Format.FractionalSeconds;
			case '.': break;
			case '3': break;
			case '6': break;
			case '9': break;

			// Hour Minute Second
			case 'R': return Format.HourMinute;
			case 'X':
			case 'T': return Format.HourMinuteSecond_24;
			case 'r': return Format.HourMinuteSecond_12;

			// Time Zone Specifiers
			case 'Z': return Format.LocalTimeZoneName;
			case 'z': return Format.OffsetFromLocaltimeFull;
			case ':': return Format.OffsetFromLocaltimeColon;
			case '#': return Format.OffsetFromLocaltimeHour;

			// Date & Time Specifiers
			case 'c': return Format.DateAndTime;
			case '+': return Format.DateAndTime_ISO_8601_RFC_3339;

			case 's': return Format.UnixTimestamp;

			// Special Specifiers
			case 't': return Format.LiteralTab;
			case 'n': return Format.LiteralNewLine;
			case '%': return Format.LiteralPercentSign;

			// Padding
			case '_': break;
			case '0': break;

			default:
				console.log('Unknown Char: ', nextChar);
				break
		}

		return null;
	}


	nextChar(): Nullable<string> {
		if (this.hasNextChar()) {
			return this.format.charAt(this.pos++);
		} else {
			return null;
		}
	}

	hasNextChar(): boolean {
		return this.pos < this.format.length;
	}
}


export {
	parseFromString
};