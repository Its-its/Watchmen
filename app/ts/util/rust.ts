
// None = "None"
// Regex(String) = "Regex": "^([a-z0-9:/.-]+)"
// TimeFormat(String, String) = "TimeFormat": [ "%b-%e %R", "PST" ]
// Test { a: String } = "Test": { a: "" }


type UpdatedValue = rust.EnumObject | rust.Values | rust.ObjectType | RustEnum | (rust.EnumValue | RustEnum)[];

export class RustEnum {
	name: string;
	value: UpdatedValue;

	constructor(name: null | string | rust.EnumObject, value?: rust.EnumValue) {
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

		this.value = genValue(this.value);
	}

	toJSON() {
		return from_rust_enum_to_object(this);
	}
}

function genValue(value: UpdatedValue): any {
	if (value == null) {
		return null;
	} else if (Array.isArray(value)) {
		return value.map(genValue);
	} else if (value instanceof RustEnum) {
		return value;
	} else if (typeof value == 'object') {
		let keys = Object.keys(value);

		if (keys.length == 1) {
			let objValue = value[keys[0]];

			return new RustEnum(keys[0], objValue);
		} else {
			return value;
		}
	} else if (typeof value == 'string' || typeof value == 'number' || typeof value == 'boolean') {
		return value;
	} else {
		return null;
	}
}


export const NULL_ENUM = () => new RustEnum('None', null);

export function rustify_object(obj: any): any {
	if (typeof obj != 'object') {
		return obj;
	}

	let corrected: rust.ObjectType = {};

	for (const key in obj) {
		if (obj.hasOwnProperty(key)) {
			const value = obj[key];

			if (value instanceof RustEnum) {
				corrected[key] = from_rust_enum_to_object(value);
			} else if (value == null) {
				corrected[key] = null;
			} else {
				corrected[key] = rustify_object(value);
			}
		}
	}

	return corrected;
}

export function from_rust_enum_to_object(obj: Nullable<RustEnum>): any {
	if (obj == null || obj.name == 'None') return 'None';

	if (obj.value == null) {
		return { [obj.name]: 'None' };
	}

	if (Array.isArray(obj.value)) {
		// if (obj.value.length == 1) {
			// return { [obj.name]: obj.value[0] };
		// } else {
			return { [obj.name]: obj.value.map(from_enum_value_to_object) };
		// }
	}

	return { [obj.name]: obj.value };
}

export function from_enum_value_to_object(value: UpdatedValue): any {
	if (value == null) return 'None';

	if (Array.isArray(value)) {
		return value.map(genValue);
	}

	if (value instanceof RustEnum) {
		return from_rust_enum_to_object(value);
	}

	return value;
}