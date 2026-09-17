import { invoke } from '$lib/invoke';
import type {
	ListedSyncProfile,
	PendingSyncConfigUpdate,
	SyncConfigFileInfo,
	SyncImportData,
	SyncPublishMode,
	SyncPullReport,
	SyncUser
} from '$lib/types';

export const read = (id: string) => invoke<SyncImportData>('read_sync_profile', { id });
export const create = () => invoke<string>('create_sync_profile');
export const push = () => invoke('push_sync_profile');
export const publish = (mode: SyncPublishMode) => invoke('publish_sync_profile', { mode });
export const getConfigFiles = () => invoke<SyncConfigFileInfo[]>('get_sync_config_files');
export const clone = (id: string, name: string) => invoke('clone_sync_profile', { id, name });
export const disconnect = (del: boolean) => invoke('disconnect_sync_profile', { delete: del });
export const deleteProfile = (id: string) => invoke('delete_sync_profile', { id });
export const pull = () => invoke<SyncPullReport>('pull_sync_profile');
export const fetch = () => invoke('fetch_sync_profile');
export const getPendingConfig = () => invoke<PendingSyncConfigUpdate[]>('get_pending_sync_config');
export const declineConfig = (files: string[]) => invoke('decline_sync_config', { files });
export const applyConfig = (files: string[]) => invoke<string[]>('apply_sync_config', { files });
export const getOwned = () => invoke<ListedSyncProfile[]>('get_owned_sync_profiles');
export const login = () => invoke<SyncUser>('login');
export const logout = () => invoke('logout');
export const getUser = () => invoke<SyncUser | null>('get_user');
