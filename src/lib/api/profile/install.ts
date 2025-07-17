import { invoke } from '$lib/invoke';
import type { ModId } from '$lib/types';

export const mod = (id: ModId) => invoke('install_mod', { id });
export const cancel = () => invoke('cancel_install');
export const clearDownloadCache = (soft: boolean) =>
	invoke<number>('clear_download_cache', { soft });
export const getDownloadSize = (modId: ModId) =>
	invoke<number>('get_download_size', { modRef: modId });
export const hasPendingInstallations = () => invoke<boolean>('has_pending_installations');
