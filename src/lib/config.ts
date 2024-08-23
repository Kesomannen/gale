import { invokeCommand } from './invoke';
import type { ConfigEntryId, ConfigValue, LoadFileResult } from './models';

export async function setConfigEntry(id: ConfigEntryId, value: ConfigValue) {
	if (
		(value.type === 'int32' || value.type === 'double' || value.type === 'single') &&
		value.content.value === null
	)
		return;

	await invokeCommand('set_config_entry', {
		file: id.file.name,
		section: id.section.name,
		entry: id.entry.name,
		value
	});

	id.entry.value = value;
}

export function configDisplayName(configFile: LoadFileResult) {
	let name: string;
	if (configFile.type == 'ok') {
		name = configFile.metadata?.pluginName ?? configFile.name;
	} else {
		name = configFile.name;
	}

	// remove underscores, hyphens, and spaces
	return name.replace(/[-_ ]/g, '');
}
