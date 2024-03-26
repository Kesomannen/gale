import { invoke } from '@tauri-apps/api';
import { writable, type Writable } from 'svelte/store';

const errorDuration = 7500;
const maxErrors = 5;

interface Error { name: string, message: string }

export const errors: Writable<Error[]> = writable([]);

export async function invokeCommand<T>(cmd: string, args?: any | undefined): Promise<T> {
	try {
		let result = await invoke<T>(cmd, args);
		return result;
	} catch (error: any) {
		errors.update((errs) => {
			let errStr = error as string;
			let message = errStr[0].toUpperCase() + errStr.slice(1) + '.';
			errs.push({ 
				name: cmd,
				message: message
			});
			if (errs.length > maxErrors) {
				errs.shift();
			}
			return errs;
		});

		setTimeout(() => {
			errors.update((errs) => {
				errs.shift();
				return errs;
			});
		}, errorDuration);

		throw error;
	}
}

export function removeError(index: number) {
	errors.update((errs) => {
		errs.splice(index, 1);
		return errs;
	});
}