import { invoke } from '$lib/invoke';
import type {
	DedicatedServerStatus,
	ProfileServerSettings,
	RemoteConnectionTestResult,
	RemoteDeploymentPreviewResult,
	RemoteDeploymentResult,
	RemoteServerSettings
} from '$lib/types';

export const getSettings = () =>
	invoke<ProfileServerSettings | null>('get_dedicated_server_settings');

export const setSettings = (settings: ProfileServerSettings) =>
	invoke('set_dedicated_server_settings', { settings });

/// `settings: null` launches with the profile's stored settings, so an
/// already-configured server starts immediately. `rememberPassword` controls
/// whether the provided (or stored) password stays in the credential store.
export const launch = (
	settings: ProfileServerSettings | null,
	password: string,
	rememberPassword: boolean
) =>
	invoke<DedicatedServerStatus>('launch_dedicated_server', {
		request: { settings, password, rememberPassword }
	});

export const testRemoteConnection = (
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) =>
	invoke<RemoteConnectionTestResult>('test_remote_server_connection', {
		request: { settings, password, rememberPassword }
	});

export const deployRemote = (
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) =>
	invoke<RemoteDeploymentResult>('deploy_remote_server', {
		request: { settings, password, rememberPassword }
	});

export const previewRemoteDeployment = (
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) =>
	invoke<RemoteDeploymentPreviewResult>('preview_remote_server_deployment', {
		request: { settings, password, rememberPassword }
	});

export const getStatus = () => invoke<DedicatedServerStatus>('get_dedicated_server_status');

export const openDir = () => invoke('open_dedicated_server_dir');

export const forceStop = () => invoke('force_stop_dedicated_server');
