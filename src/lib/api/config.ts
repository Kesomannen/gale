import { invoke } from '$lib/invoke';
import type { BaseConfigFile, ConfigEntryId, ConfigFile, ConfigValue } from '$lib/types';

const idToArgs = (id: ConfigEntryId) => ({
	file: id.file.relativePath,
	section: id.section.name,
	entry: id.entry.name
});

export const getFiles = (profileId: number) =>
	invoke<ConfigFile[]>('get_config_files', { profileId });
export const setEntry = (id: ConfigEntryId, value: ConfigValue, profileId: number) =>
	invoke('set_config_entry', {
		...idToArgs(id),
		value,
		profileId
	});
export const resetEntry = (id: ConfigEntryId, profileId: number) =>
	invoke<ConfigValue>('reset_config_entry', {
		...idToArgs(id),
		profileId
	});
export const resetAll = (file: BaseConfigFile, profileId: number) =>
	invoke('reset_config_file', {
		file: file.relativePath,
		profileId
	});
export const openFile = (file: BaseConfigFile, profileId: number) =>
	invoke('open_config_file', { file: file.relativePath, profileId });
export const deleteFile = (file: BaseConfigFile, profileId: number) =>
	invoke('delete_config_file', { file: file.relativePath, profileId });
