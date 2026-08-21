import { LazyStore } from '@tauri-apps/plugin-store';

const uiStore = new LazyStore('ui-state.json', { autoSave: true });

type Serializer<T> = {
	serialize: (value: T) => string;
	deserialize: (value: string) => T | undefined;
};

type PersistedStateOptions<T> = {
	/**
	 * The serializer to use.
	 *
	 * @default { serialize: JSON.stringify, deserialize: JSON.parse }
	 */
	serializer?: Serializer<T>;
};

const DEFAULT_SERIALIZER: Serializer<unknown> = {
	serialize: JSON.stringify,
	deserialize: JSON.parse
};

const instances = new Map<string, PersistedState<unknown>[]>();

// Writes are gated on this so the constructor's eager effect cannot overwrite
// persisted values with the initial defaults before the store has been read.
let loaded = false;

async function load(): Promise<void> {
	try {
		await uiStore.init();
		for (const [key, value] of await uiStore.entries<string>()) {
			for (const instance of instances.get(key) ?? []) {
				instance.hydrate(value);
			}
		}
		loaded = true;
	} catch (error) {
		console.error('Error when loading persisted store', error);
	}
}

void load();

/**
 * Reactive state persisted to the Tauri store (`ui-state.json`).
 *
 * Reads return the initial value until the store has loaded, then reactively swap to the persisted value. Every change (including deep mutations of nested state) is written back to the store.
 */
export class PersistedState<T> {
	#value = $state<T>() as T;
	#key: string;
	#serializer: Serializer<T>;

	constructor(key: string, initialValue: T, options: PersistedStateOptions<T> = {}) {
		this.#key = key;
		this.#serializer = options.serializer ?? (DEFAULT_SERIALIZER as Serializer<T>);
		this.#value = initialValue;

		const list = instances.get(key) ?? [];
		list.push(this as PersistedState<unknown>);
		instances.set(key, list);

		$effect.root(() => {
			$effect(() => {
				// Reading the value unconditionally keeps the effect subscribed;
				// gating only the write prevents clobbering persisted values
				// with the initial defaults before the store has been read.
				const value = this.#value;
				if (loaded) {
					void uiStore.set(this.#key, this.#serializer.serialize(value));
				}
			});
		});
	}

	get current(): T {
		return this.#value;
	}

	set current(value: T) {
		this.#value = value;
	}

	hydrate(value: string): void {
		const parsed = this.#deserialize(value);
		if (parsed !== undefined) {
			this.#value = parsed;
		}
	}

	#deserialize(value: string): T | undefined {
		try {
			return this.#serializer.deserialize(value);
		} catch (error) {
			console.error(`Error when parsing persisted store value for "${this.#key}"`, error);
		}
	}
}
