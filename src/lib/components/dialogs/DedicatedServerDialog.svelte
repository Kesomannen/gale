<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';
	import PathField from '$lib/components/ui/PathField.svelte';
	import DeploymentPreviewDialog from './DeploymentPreviewDialog.svelte';
	import DeploymentResultDialog from './DeploymentResultDialog.svelte';
	import * as api from '$lib/api';
	import type {
		ProfileServerSettings,
		RemoteAuthentication,
		RemoteConnectionTestResult,
		RemoteDeploymentPreviewResult,
		RemoteDeploymentProgress,
		RemoteDeploymentResult,
		RemoteProtocol,
		RemoteServerSettings,
		ServerLocation
	} from '$lib/types';
	import games from '$lib/state/game.svelte';
	import { Progress, Tabs } from 'bits-ui';
	import { confirm, message, open as openDialog } from '@tauri-apps/plugin-dialog';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { pushInfoToast } from '$lib/toast';

	const DEFAULT_SFTP_PORT = '22';
	const DEFAULT_FTP_PORT = '21';
	const FALLBACK_SERVER_PORT = 2456;
	const MAX_PORT = 65535;

	type Props = { open?: boolean };
	let { open = $bindable(false) }: Props = $props();

	let location = $state<ServerLocation>('local');
	let serverName = $state('');
	let worldName = $state('');
	let gamePassword = $state('');
	let rememberGamePassword = $state(true);
	let port = $state('');
	let publicServer = $state(true);
	let crossplay = $state(false);
	let extraArgs = $state('');
	let remoteHost = $state('');
	let remoteProtocol = $state<RemoteProtocol>('sftp');
	let remotePort = $state(DEFAULT_SFTP_PORT);
	let remoteUser = $state('');
	let remotePath = $state('');
	let remoteAuthentication = $state<RemoteAuthentication>('password');
	let privateKeyPath = $state('');
	let trustedHostKey = $state<string | null>(null);
	let trustedInvalidCertificateHost = $state<string | null>(null);
	let remotePassword = $state('');
	let rememberRemotePassword = $state(true);
	let initialized = $state(false);
	let loadingSettings = $state(false);
	let saving = $state(false);
	let launching = $state(false);
	let testing = $state(false);
	let deploying = $state(false);
	let deploymentProgress = $state<RemoteDeploymentProgress | null>(null);
	let unlistenProgress: UnlistenFn | null = null;
	let deploymentPreviewDialog: DeploymentPreviewDialog;
	let deploymentResultDialog: DeploymentResultDialog;

	onMount(async () => {
		unlistenProgress = await listen<RemoteDeploymentProgress>(
			'server_deployment_progress',
			(event) => {
				if (deploying) deploymentProgress = event.payload;
			}
		);
	});

	onDestroy(() => unlistenProgress?.());

	$effect(() => {
		if (!open) {
			initialized = false;
			gamePassword = '';
			remotePassword = '';
			return;
		}
		if (!initialized) void loadSettings();
	});

	/// Initial settings built from the active game, for profiles that have
	/// never configured a dedicated server.
	function defaultSettings(): ProfileServerSettings {
		const game = games.active;
		return {
			location: 'local',
			serverName: game ? `${game.name} Server` : 'Dedicated Server',
			world: game?.slug === 'valheim' ? 'Dedicated' : '',
			port: game?.dedicatedServer?.defaultPort || FALLBACK_SERVER_PORT,
			publicServer: true,
			crossplay: false,
			extraArgs: '',
			remote: {
				protocol: 'sftp',
				host: '',
				port: Number(DEFAULT_SFTP_PORT),
				username: '',
				serverDirectory: '',
				authentication: 'password',
				privateKeyPath: '',
				trustedHostKey: null,
				trustedInvalidCertificateHost: null
			}
		};
	}

	async function loadSettings() {
		initialized = true;
		loadingSettings = true;
		try {
			const value = (await api.profile.server.getSettings()) ?? defaultSettings();
			location = value.location;
			serverName = value.serverName;
			worldName = value.world;
			port = String(
				value.port || games.active?.dedicatedServer?.defaultPort || FALLBACK_SERVER_PORT
			);
			publicServer = value.publicServer;
			crossplay = value.crossplay;
			extraArgs = value.extraArgs;
			remoteHost = value.remote.host;
			remoteProtocol = value.remote.protocol === 'ftps' ? 'ftp' : value.remote.protocol;
			remotePort = String(
				value.remote.port || (remoteProtocol === 'sftp' ? DEFAULT_SFTP_PORT : DEFAULT_FTP_PORT)
			);
			remoteUser = value.remote.username;
			remotePath = value.remote.serverDirectory;
			remoteAuthentication = value.remote.authentication;
			privateKeyPath = value.remote.privateKeyPath;
			trustedHostKey = value.remote.trustedHostKey;
			trustedInvalidCertificateHost = value.remote.trustedInvalidCertificateHost;
		} finally {
			loadingSettings = false;
		}
	}

	function parsePort(value: string, label: string) {
		const parsed = Number.parseInt(value, 10);
		if (!Number.isInteger(parsed) || parsed < 1 || parsed > MAX_PORT)
			throw new Error(m.dedicatedServerDialog_portError({ label }));
		return parsed;
	}

	function remoteSettings(): RemoteServerSettings {
		return {
			protocol: remoteProtocol,
			host: remoteHost.trim(),
			port: parsePort(
				remotePort,
				remoteProtocol === 'sftp'
					? m.dedicatedServerDialog_sshPort()
					: m.dedicatedServerDialog_ftpPort()
			),
			username: remoteUser.trim(),
			serverDirectory: remotePath.trim(),
			authentication: remoteAuthentication,
			privateKeyPath: privateKeyPath.trim(),
			trustedHostKey,
			trustedInvalidCertificateHost
		};
	}

	function changeRemoteProtocol(value: RemoteProtocol) {
		if (
			(remoteProtocol === 'sftp' && remotePort === DEFAULT_SFTP_PORT) ||
			(remoteProtocol !== 'sftp' && remotePort === DEFAULT_FTP_PORT)
		) {
			remotePort = value === 'sftp' ? DEFAULT_SFTP_PORT : DEFAULT_FTP_PORT;
		}
		remoteProtocol = value;
		trustedHostKey = null;
		trustedInvalidCertificateHost = null;
	}

	async function choosePrivateKey() {
		const selected = await openDialog({
			title: m.dedicatedServerDialog_privateKeyTitle(),
			directory: false,
			multiple: false
		});
		if (typeof selected === 'string') privateKeyPath = selected;
	}

	function settings(): ProfileServerSettings {
		return {
			location,
			serverName: serverName.trim(),
			world: worldName.trim(),
			port: parsePort(port, m.dedicatedServerDialog_serverPort()),
			publicServer,
			crossplay,
			extraArgs: extraArgs.trim(),
			remote: remoteSettings()
		};
	}

	async function checkedSettings() {
		try {
			return settings();
		} catch (error) {
			await message(error instanceof Error ? error.message : String(error));
			return null;
		}
	}

	async function trustHost(fingerprint: string) {
		const accepted = await confirm(m.dedicatedServerDialog_trustMessage({ fingerprint }), {
			title: m.dedicatedServerDialog_trustTitle(),
			kind: 'warning'
		});
		if (accepted) trustedHostKey = fingerprint;
		return accepted;
	}

	async function trustInvalidCertificate() {
		const host = remoteHost.trim();
		const accepted = await confirm(m.dedicatedServerDialog_certificateMessage({ host }), {
			title: m.dedicatedServerDialog_certificateTitle(),
			kind: 'warning'
		});
		if (accepted) trustedInvalidCertificateHost = host;
		return accepted;
	}

	type TrustableResult =
		| RemoteConnectionTestResult
		| RemoteDeploymentPreviewResult
		| RemoteDeploymentResult;

	async function requestWithTrust<T extends TrustableResult>(
		settings: RemoteServerSettings,
		request: () => Promise<T>
	) {
		const result = await request();
		if (result.status === 'hostKeyUntrusted') {
			if (!(await trustHost(result.fingerprint))) return null;
			settings.trustedHostKey = trustedHostKey;
			return request();
		}
		if (result.status === 'certificateUntrusted') {
			if (!(await trustInvalidCertificate())) return null;
			settings.trustedInvalidCertificateHost = trustedInvalidCertificateHost;
			return request();
		}
		return result;
	}

	async function save() {
		const current = await checkedSettings();
		if (!current) return;
		saving = true;
		try {
			await api.profile.server.setSettings(current);
			pushInfoToast({ message: m.dedicatedServerDialog_saved() });
		} finally {
			saving = false;
		}
	}

	async function launch() {
		const current = await checkedSettings();
		if (!current) return;
		launching = true;
		try {
			await api.profile.server.launch(current, gamePassword, rememberGamePassword);
			open = false;
		} finally {
			launching = false;
		}
	}

	async function testConnection() {
		const current = await checkedSettings();
		if (!current) return;
		testing = true;
		try {
			const result = await requestWithTrust(current.remote, () =>
				api.profile.server.testRemoteConnection(
					current.remote,
					remotePassword,
					rememberRemotePassword
				)
			);
			if (!result || result.status !== 'connected') return;
			await message(
				!result.encrypted
					? m.dedicatedServerDialog_connectionPlain({ host: remoteHost })
					: remoteProtocol === 'ftp' && trustedInvalidCertificateHost === remoteHost.trim()
						? m.dedicatedServerDialog_connectionEncrypted({ host: remoteHost })
						: m.dedicatedServerDialog_connectionSecure({ host: remoteHost }),
				{
					title: m.dedicatedServerDialog_connectionTitle(),
					kind: 'info'
				}
			);
		} finally {
			testing = false;
		}
	}

	async function deploy() {
		const current = await checkedSettings();
		if (!current) return;
		deploying = true;
		deploymentProgress = null;
		try {
			const preview = await requestWithTrust(current.remote, () =>
				api.profile.server.previewRemoteDeployment(
					current.remote,
					remotePassword,
					rememberRemotePassword
				)
			);
			if (!preview || preview.status !== 'preview') return;
			if (preview.uploadFiles.length === 0 && preview.removeFiles.length === 0) {
				await message(m.dedicatedServerDialog_nothingMessage({ count: preview.unchangedFiles }), {
					title: m.dedicatedServerDialog_nothingTitle(),
					kind: 'info'
				});
				return;
			}
			if (!(await deploymentPreviewDialog.openFor(preview))) return;

			const result = await requestWithTrust(current.remote, () =>
				api.profile.server.deployRemote(current.remote, remotePassword, rememberRemotePassword)
			);
			if (!result) return;
			if (result.status === 'deployed') deploymentResultDialog.openFor(result);
		} finally {
			deploying = false;
			deploymentProgress = null;
		}
	}
</script>

<Dialog title={m.dedicatedServerDialog_title()} bind:open large>
	<p class="text-primary-600 dark:text-primary-300 mt-1">
		{m.dedicatedServerDialog_content()}
	</p>

	{#if loadingSettings}
		<div class="text-primary-500 mt-5">{m.dedicatedServerDialog_loading()}</div>
	{:else}
		<TabsMenu
			bind:value={location}
			options={[
				{ value: 'local', label: m.dedicatedServerDialog_locationLocal() },
				{ value: 'remote', label: m.dedicatedServerDialog_locationRemote() }
			]}
		>
			<Tabs.Content value="local">
				<div class="mt-4 flex flex-col gap-3">
					<div>
						<Label>{m.dedicatedServerDialog_serverName()}</Label><InputField
							class="mt-1 w-full"
							bind:value={serverName}
							placeholder={m.dedicatedServerDialog_serverNamePlaceholder()}
						/>
					</div>
					<div>
						<Label>{m.dedicatedServerDialog_world()}</Label><InputField
							class="mt-1 w-full"
							bind:value={worldName}
							placeholder={m.dedicatedServerDialog_worldPlaceholder()}
						/>
					</div>
					<div>
						<Label>{m.dedicatedServerDialog_password()}</Label><InputField
							class="mt-1 w-full"
							bind:value={gamePassword}
							type="password"
						/>
						<p class="text-primary-500 mt-1 text-sm">{m.dedicatedServerDialog_savedPassword()}</p>
					</div>
					<div class="flex items-center">
						<Label>{m.dedicatedServerDialog_rememberPassword()}</Label><Info
							>{m.dedicatedServerDialog_credentialInfo()}</Info
						><Checkbox bind:checked={rememberGamePassword} />
					</div>
					<div>
						<Label>{m.dedicatedServerDialog_serverPort()}</Label><InputField
							class="mt-1 w-full"
							bind:value={port}
							inputmode="numeric"
						/>
					</div>
					<div class="flex items-center">
						<Label>{m.dedicatedServerDialog_public()}</Label><Info
							>{m.dedicatedServerDialog_publicInfo()}</Info
						><Checkbox bind:checked={publicServer} />
					</div>
					<div class="flex items-center">
						<Label>{m.dedicatedServerDialog_crossplay()}</Label><Info
							>{m.dedicatedServerDialog_crossplayInfo()}</Info
						><Checkbox bind:checked={crossplay} />
					</div>
				</div>
			</Tabs.Content>

			<Tabs.Content value="remote">
				<div class="mt-4 flex flex-col gap-3">
					<InfoBox type={remoteProtocol === 'ftp' ? 'warning' : 'info'}
						>{remoteProtocol === 'ftp'
							? m.dedicatedServerDialog_ftpInfo()
							: m.dedicatedServerDialog_sftpInfo()}</InfoBox
					>
					<div>
						<Label>{m.dedicatedServerDialog_protocol()}</Label>
						<Select
							type="single"
							triggerClass="mt-1 w-full"
							bind:value={remoteProtocol}
							onValueChange={(value) => changeRemoteProtocol(value as RemoteProtocol)}
							items={[
								{ value: 'sftp', label: m.dedicatedServerDialog_protocolSftp() },
								{ value: 'ftp', label: m.dedicatedServerDialog_protocolFtp() }
							]}
						/>
					</div>
					<div>
						<Label>{m.dedicatedServerDialog_host()}</Label><InputField
							class="mt-1 w-full"
							bind:value={remoteHost}
							placeholder="example.com"
						/>
					</div>
					<div class="grid grid-cols-2 gap-3">
						<div>
							<Label
								>{remoteProtocol === 'sftp'
									? m.dedicatedServerDialog_sshPort()
									: m.dedicatedServerDialog_ftpPort()}</Label
							><InputField class="mt-1 w-full" bind:value={remotePort} inputmode="numeric" />
						</div>
						<div>
							<Label>{m.dedicatedServerDialog_username()}</Label><InputField
								class="mt-1 w-full"
								bind:value={remoteUser}
							/>
						</div>
					</div>
					{#if remoteProtocol === 'sftp'}
						<div>
							<Label>{m.dedicatedServerDialog_authentication()}</Label>
							<Select
								type="single"
								triggerClass="mt-1 w-full"
								bind:value={remoteAuthentication}
								items={[
									{ value: 'password', label: m.dedicatedServerDialog_password() },
									{ value: 'privateKey', label: m.dedicatedServerDialog_privateKeyFile() },
									{ value: 'agent', label: m.dedicatedServerDialog_sshAgent() }
								]}
							/>
						</div>
					{/if}
					{#if remoteProtocol === 'sftp' && remoteAuthentication === 'privateKey'}
						<PathField
							label={m.dedicatedServerDialog_privateKey()}
							bind:value={privateKeyPath}
							onclick={choosePrivateKey}
							icon="mdi:file-key"
						>
							{m.dedicatedServerDialog_privateKeyInfo()}
						</PathField>
					{/if}
					{#if remoteProtocol !== 'sftp' || remoteAuthentication !== 'agent'}
						<div>
							<Label
								>{remoteProtocol !== 'sftp' || remoteAuthentication === 'password'
									? m.dedicatedServerDialog_password()
									: m.dedicatedServerDialog_keyPassphrase()}</Label
							>
							<InputField class="mt-1 w-full" bind:value={remotePassword} type="password" />
							<p class="text-primary-500 mt-1 text-sm">
								{remoteProtocol !== 'sftp' || remoteAuthentication === 'password'
									? m.dedicatedServerDialog_savedPassword()
									: m.dedicatedServerDialog_remoteSavedPassphrase()}
							</p>
						</div>
						<div class="flex items-center">
							<Label
								>{remoteProtocol !== 'sftp' || remoteAuthentication === 'password'
									? m.dedicatedServerDialog_rememberPassword()
									: m.dedicatedServerDialog_rememberPassphrase()}</Label
							>
							<Info>{m.dedicatedServerDialog_credentialInfo()}</Info>
							<Checkbox bind:checked={rememberRemotePassword} />
						</div>
					{/if}
					<div>
						<Label>{m.dedicatedServerDialog_directory()}</Label><InputField
							class="mt-1 w-full"
							bind:value={remotePath}
							placeholder="/home/valheim/server"
						/>
					</div>
					<div>
						<Button
							color="primary"
							icon="mdi:lan-connect"
							loading={testing}
							onclick={testConnection}>{m.dedicatedServerDialog_test()}</Button
						>
					</div>
					{#if deploying && deploymentProgress}
						<div class="flex flex-col gap-1">
							<div
								class="text-primary-600 dark:text-primary-300 flex justify-between gap-3 text-sm"
							>
								<span class="truncate">
									{deploymentProgress.operation === 'upload'
										? m.dedicatedServerDialog_progressUpload({ path: deploymentProgress.path })
										: m.dedicatedServerDialog_progressRemove({ path: deploymentProgress.path })}
								</span>
								<span class="shrink-0"
									>{deploymentProgress.completed}/{deploymentProgress.total}</span
								>
							</div>
							<Progress.Root
								class="bg-primary-200 dark:bg-primary-700 h-2 overflow-hidden rounded-full"
								value={deploymentProgress.completed}
								max={deploymentProgress.total}
							>
								<div
									class="bg-accent-600 dark:bg-accent-500 h-full transition-[width]"
									style="width: {(deploymentProgress.completed / deploymentProgress.total) * 100}%"
								></div>
							</Progress.Root>
						</div>
					{/if}
				</div>
			</Tabs.Content>
		</TabsMenu>

		<details class="mt-4">
			<summary class="text-primary-600 dark:text-primary-300 cursor-pointer"
				>{m.dedicatedServerDialog_advancedOptions()}</summary
			>
			<div class="mt-2">
				<Label>{m.dedicatedServerDialog_additionalArgs()}</Label><InputField
					class="mt-1 w-full"
					bind:value={extraArgs}
					placeholder="-savedir ..."
				/>
			</div>
		</details>
	{/if}

	<div class="mt-5 flex w-full items-center justify-end gap-2">
		<Button color="primary" onclick={() => (open = false)}
			>{m.dedicatedServerDialog_cancel()}</Button
		>
		<Button color="primary" icon="mdi:content-save" loading={saving} onclick={save}
			>{m.dedicatedServerDialog_save()}</Button
		>
		{#if location === 'local'}
			<Button icon="mdi:server" loading={launching} onclick={launch}
				>{m.dedicatedServerDialog_launch()}</Button
			>
		{:else}
			<Button icon="mdi:cloud-upload" loading={deploying} onclick={deploy}
				>{m.dedicatedServerDialog_deploy()}</Button
			>
		{/if}
	</div>
</Dialog>

<DeploymentPreviewDialog bind:this={deploymentPreviewDialog} />
<DeploymentResultDialog bind:this={deploymentResultDialog} />
