import { invoke } from '@tauri-apps/api/core';
import { writable, type Writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

const errorDuration = 10000;
const maxErrors = 10;

interface InvokeError { name: string, message: string }

listen<InvokeError>('error', (evt) => pushError(evt.payload));

export const errors: Writable<InvokeError[]> = writable([]);

export async function invokeCommand<T>(cmd: string, args?: any): Promise<T> {
	try {
		return await invoke<T>(cmd, args);
	} catch (error: any) {
		let errStr = error as string;
		let name = `Failed to execute '${cmd}'`
		let message = errStr[0].toUpperCase() + errStr.slice(1) + '.';
		
		pushError({ name, message }, false);
		throw error;
	}
}

export function pushError(error: InvokeError, throwErr: boolean = true) {
	errors.update((errs) => {
		errs.push(error);
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

	let msg = `${error.name}: ${error.message}`;
	invoke('log_err', { msg })

	if (throwErr) {
		throw new Error(msg);
	}
}

export function removeError(index: number) {
	errors.update((errs) => {
		errs.splice(index, 1);
		return errs;
	});
}
