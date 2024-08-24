import { invokeCommand } from './invoke';
import type { ConfigEntryId, ConfigValue, LoadFileResult } from './models';

export async function setConfigEntry(id: ConfigEntryId, value: ConfigValue) {
	if (
		(value.type === 'int32' || value.type === 'double' || value.type === 'single') &&
		value.content.value === null
	)
		return;

	await invokeCommand('set_config_entry', {
		file: id.file.relativePath,
		section: id.section.name,
		entry: id.entry.name,
		value
	});

	id.entry.value = value;
}
