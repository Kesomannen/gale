import { writable, type Writable } from 'svelte/store';

const errorDuration = 8000;
const infoDuration = 3000;
const maxCount = 5;

export type Toast = {
	type: 'error' | 'info';
	name?: string;
	message: string;
};

export const toasts: Writable<(Toast & { id: number })[]> = writable([]);

export function pushInfoToast(toast: { name?: undefined; message: string }) {
	pushToast({
		type: 'info',
		...toast
	});
}

let nextId = 0;

export function pushToast(toast: Toast) {
	toasts.update((toasts) => {
		toasts.push({ ...toast, id: nextId });
		nextId++;
		if (toasts.length > maxCount) {
			toasts.shift();
		}
		return toasts;
	});

	setTimeout(
		() => {
			toasts.update((toasts) => {
				toasts.shift();
				return toasts;
			});
		},
		toast.type === 'error' ? errorDuration : infoDuration
	);
}

export function clearToast(index: number) {
	toasts.update((toasts) => {
		toasts.splice(index, 1);
		return toasts;
	});
}
