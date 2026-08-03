import { invoke } from '$lib/invoke';
import type { ModId } from '$lib/types';

export const changeModVersion = (id: ModId) => invoke('change_mod_version', { id });
export const mods = (packageUuids: string[], respectIgnored: boolean) =>
	invoke('update_mods', { uuids: packageUuids, respectIgnored });
export const ignore = (versionUuid: string) => invoke('ignore_update', { versionUuid });
export const ignorePackage = (packageUuid: string) =>
	invoke('ignore_package_updates', { packageUuid });
