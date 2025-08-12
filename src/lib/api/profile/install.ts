import { invoke } from '$lib/invoke';
import type { ModId } from '$lib/types';

export const allMods = () => invoke('install_all_mods');
export const mod = (id: ModId) => invoke('install_mod', { id });
export const cancelAll = () => invoke('cancel_all_installs');
export const clearDownloadCache = (soft: boolean) =>
	invoke<number>('clear_download_cache', { soft });
export const getDownloadSize = (modId: ModId) =>
	invoke<number>('get_download_size', { modRef: modId });
export const hasPendingInstallations = () => invoke<boolean>('has_pending_installations');
