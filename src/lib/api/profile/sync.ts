import { invoke } from '$lib/invoke';
import type {
	ListedSyncProfile,
	SyncConfigApplyReport,
	SyncConfigFileInfo,
	SyncConfigReviewState,
	SyncConfigUpdatePolicy,
	SyncImportData,
	SyncPublishMode,
	SyncUser
} from '$lib/types';

export const read = (id: string) => invoke<SyncImportData>('read_sync_profile', { id });
export const create = (profileId: number) => invoke<string>('create_sync_profile', { profileId });
export const push = (mode: SyncPublishMode, profileId: number) =>
	invoke('push_sync_profile', { mode, profileId });
export const getConfigFiles = (profileId: number) =>
	invoke<SyncConfigFileInfo[]>('get_sync_config_files', { profileId });
export const clone = (id: string, name: string) => invoke('clone_sync_profile', { id, name });
export const disconnect = (del: boolean, profileId: number) =>
	invoke('disconnect_sync_profile', { delete: del, profileId });
export const deleteProfile = (id: string) => invoke('delete_sync_profile', { id });
export const pull = (profileId: number) =>
	invoke<SyncConfigApplyReport>('pull_sync_profile', { profileId });
export const fetch = (profileId: number) => invoke('fetch_sync_profile', { profileId });
export const getPendingConfig = (profileId: number) =>
	invoke<SyncConfigReviewState>('get_pending_sync_config', { profileId });
export const declineConfig = (files: string[], remember: boolean, profileId: number) =>
	invoke('decline_sync_config', { files, remember, profileId });
export const applyConfig = (
	files: string[],
	remember: boolean,
	restoreDeleted: string[],
	profileId: number
) => invoke<string[]>('apply_sync_config', { files, remember, restoreDeleted, profileId });
export const setConfigPolicy = (file: string, policy: SyncConfigUpdatePolicy, profileId: number) =>
	invoke('set_sync_config_policy', { file, policy, profileId });
export const getOwned = () => invoke<ListedSyncProfile[]>('get_owned_sync_profiles');
export const login = () => invoke<SyncUser>('login');
export const logout = () => invoke('logout');
export const getUser = () => invoke<SyncUser | null>('get_user');
