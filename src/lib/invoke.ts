import { invoke } from '@tauri-apps/api';
import { writable, type Writable } from 'svelte/store';
import type { ConfigEntryId, ConfigFile, ConfigSection, ConfigValue } from './models';
import { listen } from '@tauri-apps/api/event';

const errorDuration = 7500;
const maxErrors = 5;

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

	if (throwErr) {
		throw new Error(`${error.name}: ${error.message}`);
	}
}

export function removeError(index: number) {
	errors.update((errs) => {
		errs.splice(index, 1);
		return errs;
	});
}

export function setTaggedConfig(id: ConfigEntryId, value: ConfigValue) {
	if ((value.type === 'int32' || value.type === 'double' || value.type === 'single') && !value.content.value) return;

	invokeCommand('set_tagged_config_entry', { 
		file: id.file.content.name,
		section: id.section.name,
		entry: id.entry.name,
		value 
	}).then(() => id.entry.value = value);
}

export function setUntaggedConfig(file: ConfigFile, section: ConfigSection, name: string, value: string) {
	if (!value) return;
	
	invokeCommand('set_untagged_config_entry', { 
		file: file.name,
		section: section.name,
		entry: name,
		value
	});
}
