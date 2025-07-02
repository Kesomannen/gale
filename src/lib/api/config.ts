import { invoke } from '$lib/invoke';
import type { ConfigEntryId, ConfigFile, ConfigValue } from '$lib/types';

const idToArgs = (id: ConfigEntryId) => ({
	file: id.file.relativePath,
	section: id.section.name,
	entry: id.entry.name
});

export const getFiles = () => invoke<ConfigFile[]>('get_config_files');
export const setEntry = (id: ConfigEntryId, value: ConfigValue) =>
	invoke('set_config_entry', {
		...idToArgs(id),
		value
	});
export const resetEntry = (id: ConfigEntryId) =>
	invoke<ConfigValue>('reset_config_entry', {
		...idToArgs(id)
	});
export const openFile = (file: ConfigFile) =>
	invoke('open_config_file', { file: file.relativePath });
export const deleteFile = (file: ConfigFile) =>
	invoke('delete_config_file', { file: file.relativePath });
