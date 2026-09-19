import * as api from '$lib/api';
import config from '$lib/state/config.svelte';
import type { BaseConfigFile, ConfigEntryId, ConfigValue } from './types';

export function isNum(value: ConfigValue) {
	return value.type === 'int' || value.type === 'float';
}

export async function setConfigEntry(id: ConfigEntryId, value: ConfigValue) {
	if (isNum(value) && value.content.value === null) return;
	if (config.profileId === null) return;

	await api.config.setEntry(id, value, config.profileId);

	id.entry.value = value;
}

export async function openConfigFile(file: BaseConfigFile) {
	if (config.profileId === null) return;
	await api.config.openFile(file, config.profileId);
}
