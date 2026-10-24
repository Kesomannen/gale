import { invoke } from '$lib/invoke';
import type {
	DedicatedServerStatus,
	ProfileServerSettings,
	RemoteConnectionTestResult,
	RemoteDeploymentPreviewResult,
	RemoteDeploymentResult,
	RemoteDeploymentSelection
} from '$lib/types';

export const getSettings = () =>
	invoke<ProfileServerSettings | null>('get_dedicated_server_settings');

export const setSettings = (
	settings: ProfileServerSettings,
	gamePassword: string,
	remoteCredential: string
) => invoke('set_dedicated_server_settings', { settings, gamePassword, remoteCredential });

export const launch = (settings: ProfileServerSettings) =>
	invoke<DedicatedServerStatus>('launch_dedicated_server', { settings });

export const testRemoteConnection = (settings: ProfileServerSettings) =>
	invoke<RemoteConnectionTestResult>('test_remote_server_connection', { settings });

export const deployRemote = (
	settings: ProfileServerSettings,
	selection: RemoteDeploymentSelection
) => invoke<RemoteDeploymentResult>('deploy_remote_server', { settings, selection });

export const previewRemoteDeployment = (settings: ProfileServerSettings) =>
	invoke<RemoteDeploymentPreviewResult>('preview_remote_server_deployment', { settings });

export const getStatus = () => invoke<DedicatedServerStatus>('get_dedicated_server_status');

export const openDir = () => invoke('open_dedicated_server_dir');

export const forceStop = () => invoke('force_stop_dedicated_server');
