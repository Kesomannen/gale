import { invoke } from '$lib/invoke';
import type { ModId, ModpackChange } from '$lib/types';

export const changeModVersions = (ids: ModId[]) => invoke('change_mod_versions', { ids });
export const mods = (packageUuids: string[], respectIgnored: boolean) =>
	invoke('update_mods', { uuids: packageUuids, respectIgnored });
export const ignore = (versionUuid: string) => invoke('ignore_update', { versionUuid });
export const ignorePackage = (packageUuid: string) =>
	invoke('ignore_package_updates', { packageUuid });
export const modpackChanges = (id: ModId) => invoke<ModpackChange[]>('get_modpack_changes', { id });
