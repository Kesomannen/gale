type Default<T> = T | (() => T);

function evalDefault<T>(defaultValue: Default<T>) {
	if (typeof defaultValue === 'function') {
		return (defaultValue as () => T)();
	} else {
		return defaultValue;
	}
}

function get<T>(key: string, defaultValue: Default<T>): T {
	let value = localStorage.getItem(key);
	if (value === null) return evalDefault(defaultValue);

	let deserialized = JSON.parse(value) as T | null | undefined;
	if (deserialized === null || deserialized === undefined) return evalDefault(defaultValue);

	return deserialized;
}

function set<T>(key: string, value: T) {
	localStorage.setItem(key, JSON.stringify(value));
}

export const store = {
	get,
	set
};
