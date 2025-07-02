import * as api from '$lib/api';
import type { ConfigEntryId, ConfigValue } from './types';

export function isNum(value: ConfigValue) {
	return value.type === 'int' || value.type === 'float';
}

export async function setConfigEntry(id: ConfigEntryId, value: ConfigValue) {
	if (isNum(value) && value.content.value === null) return;

	await api.config.setEntry(id, value);

	id.entry.value = value;
}
