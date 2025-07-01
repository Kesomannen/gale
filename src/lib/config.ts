import { invokeCommand } from './invoke';
import type { ConfigEntryId, ConfigValue } from './types';

export function isNum(value: ConfigValue) {
	return value.type === 'int' || value.type === 'float';
}

export async function setConfigEntry(id: ConfigEntryId, value: ConfigValue) {
	if (isNum(value) && value.content.value === null) return;

	await invokeCommand('set_config_entry', {
		file: id.file.relativePath,
		section: id.section.name,
		entry: id.entry.name,
		value
	});

	id.entry.value = value;
}
