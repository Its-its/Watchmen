type ExcludeReadonlyKeys<T> = Pick<T, { [P in keyof T]: 'readonly' extends keyof T[P] ? never : P }[keyof T]>;
type ExcludeFunctionKeys<T> = Pick<T, { [F in keyof T]: T[F] extends (...args: any) => any ? never : F }[keyof T]>;

type PartialOnlyProperties<T> = Partial<ExcludeReadonlyKeys<T> & ExcludeFunctionKeys<T>>;

export function createElement<K extends keyof HTMLElementTagNameMap>(
	tagName: K,
	props?: PartialOnlyProperties<HTMLElementTagNameMap[K]> & { [key: string]: any },
	appendTo?: HTMLElement
): HTMLElementTagNameMap[K] {
	let element = document.createElement(tagName);

	if (props !== undefined) {
		for (const key in props) {
			const value = props[key];

			// @ts-ignore
			element[key] = value;
		}
	}

	if (appendTo !== undefined) {
		appendTo.appendChild(element);
	}

	return element;
}