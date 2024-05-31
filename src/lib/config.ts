import { invokeCommand } from "./invoke";
import type { ConfigEntryId, ConfigFile, ConfigSection, ConfigValue } from "./models";

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
