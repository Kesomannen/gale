import { invoke } from '@tauri-apps/api';
import { writable, type Writable } from 'svelte/store';
import type { ConfigEntryId, ConfigValue } from './models';

const errorDuration = 7500;
const maxErrors = 5;

interface Error { name: string, message: string }

export const errors: Writable<Error[]> = writable([]);

export async function invokeCommand<T>(cmd: string, args?: any): Promise<T> {
	try {
		return await invoke<T>(cmd, args);
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

export function setConfig(id: ConfigEntryId, value: ConfigValue) {
	invokeCommand('set_config_entry', { 
		file: id.file.name,
		section: id.section.name,
		entry: id.entry.name,
		value 
	}).then(() => id.entry.value = value);
}
