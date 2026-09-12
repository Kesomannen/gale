import { invoke } from '$lib/invoke';

export type ServerLocation = 'local' | 'remote';
export type RemoteAuthentication = 'password' | 'privateKey' | 'agent';
export type RemoteProtocol = 'sftp' | 'ftp' | 'ftps';

export type RemoteServerSettings = {
	protocol: RemoteProtocol;
	host: string;
	port: number;
	username: string;
	serverDirectory: string;
	authentication: RemoteAuthentication;
	privateKeyPath: string;
	trustedHostKey: string | null;
	trustedInvalidCertificateHost: string | null;
};

export type DedicatedServerSettings = {
	location: ServerLocation;
	serverName: string;
	world: string;
	port: number;
	publicServer: boolean;
	crossplay: boolean;
	extraArgs: string;
	remote: RemoteServerSettings;
};

export type RemoteConnectionTestResult =
	| { status: 'connected'; fingerprint: string | null; encrypted: boolean }
	| { status: 'hostKeyUntrusted'; fingerprint: string }
	| { status: 'certificateUntrusted' };

export type RemoteDeploymentResult =
	| { status: 'hostKeyUntrusted'; fingerprint: string }
	| { status: 'certificateUntrusted' }
	| {
			status: 'deployed';
			fingerprint: string | null;
			uploadedFiles: number;
			uploadedBytes: number;
			removedFiles: number;
			unchangedFiles: number;
			skippedClientOnlyMods: string[];
			cleanupWarnings: string[];
	  };

export type RemoteDeploymentPreviewResult =
	| { status: 'hostKeyUntrusted'; fingerprint: string }
	| { status: 'certificateUntrusted' }
	| {
			status: 'preview';
			fingerprint: string | null;
			uploadFiles: string[];
			uploadBytes: number;
			removeFiles: string[];
			unchangedFiles: number;
			skippedClientOnlyMods: string[];
	  };

export type RemoteDeploymentProgress = {
	completed: number;
	total: number;
	path: string;
	operation: 'upload' | 'remove';
};

export type DedicatedServerStatus =
	| { state: 'stopped' }
	| {
			state: 'running';
			profileId: number;
			gameSlug: string;
			pid: number;
			serverDir: string;
	  };

export function getSettings() {
	return invoke<DedicatedServerSettings>('get_dedicated_server_settings');
}

export function setSettings(settings: DedicatedServerSettings) {
	return invoke('set_dedicated_server_settings', { settings });
}

export function launch(
	settings: DedicatedServerSettings,
	password: string,
	rememberPassword: boolean
) {
	return invoke<DedicatedServerStatus>('launch_dedicated_server', {
		request: { settings, password, rememberPassword }
	});
}

export function testRemoteConnection(
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) {
	return invoke<RemoteConnectionTestResult>('test_remote_server_connection', {
		request: { settings, password, rememberPassword }
	});
}

export function deployRemote(
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) {
	return invoke<RemoteDeploymentResult>('deploy_remote_server', {
		request: { settings, password, rememberPassword }
	});
}

export function previewRemoteDeployment(
	settings: RemoteServerSettings,
	password: string,
	rememberPassword: boolean
) {
	return invoke<RemoteDeploymentPreviewResult>('preview_remote_server_deployment', {
		request: { settings, password, rememberPassword }
	});
}

export function getStatus() {
	return invoke<DedicatedServerStatus>('get_dedicated_server_status');
}

export function openDir() {
	return invoke('open_dedicated_server_dir');
}

export function forceStop() {
	return invoke('force_stop_dedicated_server');
}
