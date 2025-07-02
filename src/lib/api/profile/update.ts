import { invoke } from '$lib/invoke';
import type { ModId } from '$lib/types';

export const changeModVersion = (id: ModId) => invoke('change_mod_version', { modRef: id });
export const mods = (uuids: string[], respectIgnored: boolean) =>
	invoke('update_mods', { uuids, respectIgnored });
export const ignore = (versionUuid: string) => invoke('ignore_update', { versionUuid });
