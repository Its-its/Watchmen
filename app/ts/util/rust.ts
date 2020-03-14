
// None = "None"
// Regex(String) = "Regex": "^([a-z0-9:/.-]+)"
// TimeFormat(String, String) = "TimeFormat": [ "%b-%e %R", "PST" ]
// Test { a: String } = "Test": { a: "" }

type Values = string | string[] | number | number[] | boolean | boolean[];
export type EnumValue = null | Values | ObjectType;
export type EnumObject = { [name: string]: EnumValue };


export class RustEnum {
	name: string;
	value: EnumValue;

	constructor(name: null | string | EnumObject, value?: EnumValue) {
		if (name == null) {
			this.name = 'None';
			this.value = null;
		} else if (typeof name == 'string') {
			this.name = name;
			this.value = value!;
		} else {
			let keys = Object.keys(name);
			this.name = keys[0];
			this.value = name[this.name];
		}
	}
}

export type ObjectType = {
	[name: string]: Values;
};



export const NULL_ENUM = () => new RustEnum('None', null);

export function rustify_object(obj: any): any {
	if (typeof obj != 'object') {
		return obj;
	}

	let corrected: ObjectType = {};

	for (const key in obj) {
		if (obj.hasOwnProperty(key)) {
			const value = obj[key];

			if (value instanceof RustEnum || value == null) {
				corrected[key] = object_to_rust_enum(value);
			} else {
				corrected[key] = rustify_object(value);
			}
		}
	}

	return corrected;
}

export function object_to_rust_enum(obj: Nullable<RustEnum>): any {
	console.log('Enum: ', obj);
	if (obj == null || obj.name == 'None') return 'None';

	if (obj.value == null) {
		return { [obj.name]: 'None' };
	}

	if (Array.isArray(obj.value)) {
		if (obj.value.length == 1) {
			return { [obj.name]: obj.value[0] };
		} else {
			return { [obj.name]: obj.value };
		}
	}

	return { [obj.name]: obj.value };
}