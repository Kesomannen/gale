import { writable, type Writable } from 'svelte/store';

const errorDuration = 8000;
const infoDuration = 3000;
const maxCount = 5;

type Toast = {
	type: 'error' | 'info';
	name?: string;
	message: string;
};

export const toasts: Writable<Toast[]> = writable([]);

export function pushInfoToast(toast: { name?: undefined; message: string }) {
	pushToast({
		type: 'info',
		...toast
	});
}

export function pushToast(toast: Toast) {
	toasts.update((toasts) => {
		toasts.push(toast);
		if (toasts.length > maxCount) {
			toasts.shift();
		}
		return toasts;
	});

	setTimeout(
		() => {
			console.log('clearing toast');
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
