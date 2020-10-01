
// None = "None"
// Regex(String) = "Regex": "^([a-z0-9:/.-]+)"
// TimeFormat(String, String) = "TimeFormat": [ "%b-%e %R", "PST" ]
// Test { a: String } = "Test": { a: "" }



export class RustEnum {
	name: string;
	value: rust.EnumValue;

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
				corrected[key] = object_to_rust_enum(value);
			} else if (value == null) {
				corrected[key] = null;
			} else {
				corrected[key] = rustify_object(value);
			}
		}
	}

	return corrected;
}

export function object_to_rust_enum(obj: Nullable<RustEnum>): any {
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