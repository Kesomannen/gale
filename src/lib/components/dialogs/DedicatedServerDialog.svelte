<script lang="ts">
	import DedicatedServerSettingsDialog from './DedicatedServerSettingsDialog.svelte';
	import DeploymentPreviewDialog from './DeploymentPreviewDialog.svelte';
	import DeploymentResultDialog from './DeploymentResultDialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { pushInfoToast } from '$lib/toast';
	import type {
		ProfileServerSettings,
		RemoteConnectionTestResult,
		RemoteDeploymentPreviewResult,
		RemoteDeploymentProgress,
		RemoteDeploymentResult
	} from '$lib/types';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { onDestroy, onMount } from 'svelte';

	type Activity = 'loading' | 'launching' | 'testing' | 'previewing' | 'deploying' | null;
	type TrustableResult =
		| RemoteConnectionTestResult
		| RemoteDeploymentPreviewResult
		| RemoteDeploymentResult;
	type Props = { open?: boolean };

	let { open = $bindable(false) }: Props = $props();
	let settings = $state<ProfileServerSettings | null>(null);
	let settingsDialogOpen = $state(false);
	let activity = $state<Activity>(null);
	let initialized = $state(false);
	let deploymentProgress = $state<RemoteDeploymentProgress | null>(null);
	let unlistenProgress: UnlistenFn | null = null;
	let deploymentPreviewDialog: DeploymentPreviewDialog;
	let deploymentResultDialog: DeploymentResultDialog;

	onMount(async () => {
		unlistenProgress = await listen<RemoteDeploymentProgress>(
			'server_deployment_progress',
			(event) => {
				if (activity === 'deploying') deploymentProgress = event.payload;
			}
		);
	});

	onDestroy(() => unlistenProgress?.());

	$effect(() => {
		if (!open) {
			initialized = false;
			return;
		}
		if (!initialized) void load();
	});

	async function load() {
		initialized = true;
		activity = 'loading';
		try {
			settings = await api.profile.server.getSettings();
			if (!settings) {
				open = false;
				settingsDialogOpen = true;
			}
		} finally {
			activity = null;
		}
	}

	function configure() {
		open = false;
		settingsDialogOpen = true;
	}

	function settingsSaved(value: ProfileServerSettings) {
		settings = value;
		initialized = true;
		open = true;
	}

	async function trustHost(fingerprint: string) {
		const accepted = await confirm(m.dedicatedServerDialog_trustMessage({ fingerprint }), {
			title: m.dedicatedServerDialog_trustTitle(),
			kind: 'warning'
		});
		if (accepted && settings) settings.remote.trustedHostKey = fingerprint;
		return accepted;
	}

	async function trustInvalidCertificate() {
		if (!settings) return false;

		const host = settings.remote.host.trim();
		const accepted = await confirm(m.dedicatedServerDialog_certificateMessage({ host }), {
			title: m.dedicatedServerDialog_certificateTitle(),
			kind: 'warning'
		});
		if (accepted) settings.remote.trustedInvalidCertificateHost = host;
		return accepted;
	}

	async function requestWithTrust<T extends TrustableResult>(request: () => Promise<T>) {
		const result = await request();
		if (result.status === 'hostKeyUntrusted') {
			if (!(await trustHost(result.fingerprint))) return null;
			return request();
		}
		if (result.status === 'certificateUntrusted') {
			if (!(await trustInvalidCertificate())) return null;
			return request();
		}
		return result;
	}

	async function launch() {
		if (!settings) return;

		activity = 'launching';
		try {
			await api.profile.server.launch(settings);
			open = false;
		} finally {
			activity = null;
		}
	}

	async function testConnection() {
		if (!settings) return;

		activity = 'testing';
		try {
			const result = await requestWithTrust(() =>
				api.profile.server.testRemoteConnection(settings!)
			);
			if (!result || result.status !== 'connected') return;

			const host = settings.remote.host;
			const message = !result.encrypted
				? m.dedicatedServerDialog_connectionPlain({ host })
				: settings.remote.protocol === 'ftp' &&
					  settings.remote.trustedInvalidCertificateHost === host.trim()
					? m.dedicatedServerDialog_connectionEncrypted({ host })
					: m.dedicatedServerDialog_connectionSecure({ host });
			pushInfoToast({ message });
		} finally {
			activity = null;
		}
	}

	async function deploy() {
		if (!settings) return;

		activity = 'previewing';
		deploymentProgress = null;
		try {
			const preview = await requestWithTrust(() =>
				api.profile.server.previewRemoteDeployment(settings!)
			);
			if (!preview || preview.status !== 'preview') return;
			const hasChanges = preview.uploadFiles.length > 0 || preview.removeFiles.length > 0;
			const hasNotices = preview.skippedClientOnlyMods.length > 0 || preview.preservedFiles > 0;
			if (!hasChanges && !hasNotices) {
				pushInfoToast({
					message: m.dedicatedServerDialog_nothingMessage({ count: preview.unchangedFiles })
				});
				return;
			}

			activity = null;
			const selection = await deploymentPreviewDialog.openFor(preview);
			if (!hasChanges || !selection) return;

			activity = 'deploying';
			const result = await requestWithTrust(() =>
				api.profile.server.deployRemote(settings!, selection)
			);
			if (result?.status === 'deployed') {
				open = false;
				deploymentResultDialog.openFor(result);
			}
		} finally {
			activity = null;
			deploymentProgress = null;
		}
	}
</script>

<Dialog title={m.dedicatedServerDialog_title()} bind:open large canClose={activity === null}>
	{#if activity === 'loading' || !settings}
		<div class="text-primary-500 dark:text-primary-400 flex justify-center py-12 text-3xl">
			<Spinner />
		</div>
	{:else}
		<p class="text-primary-600 dark:text-primary-300 mt-1">
			{settings.location === 'local'
				? m.dedicatedServerDialog_localSummary({ name: settings.local.serverName })
				: m.dedicatedServerDialog_remoteSummary({ host: settings.remote.host })}
		</p>

		{#if activity === 'deploying' && deploymentProgress}
			<div class="mt-5 flex flex-col gap-1">
				<div class="text-primary-600 dark:text-primary-300 flex justify-between gap-3 text-sm">
					<span class="truncate">
						{deploymentProgress.operation === 'upload'
							? m.dedicatedServerDialog_progressUpload({ path: deploymentProgress.path })
							: m.dedicatedServerDialog_progressRemove({ path: deploymentProgress.path })}
					</span>
					<span class="shrink-0">
						{deploymentProgress.completed}/{deploymentProgress.total}
					</span>
				</div>
				<ProgressBar value={deploymentProgress.completed} max={deploymentProgress.total} />
			</div>
		{/if}

		<div class="mt-5 flex justify-end gap-2">
			<Button color="primary" disabled={activity !== null} onclick={() => (open = false)}>
				{m.dedicatedServerDialog_cancel()}
			</Button>
			<Button color="primary" icon="mdi:cog" disabled={activity !== null} onclick={configure}>
				{m.dedicatedServerDialog_settings()}
			</Button>
			{#if settings.location === 'local'}
				<Button
					icon="mdi:server"
					loading={activity === 'launching'}
					disabled={activity !== null}
					onclick={launch}
				>
					{m.dedicatedServerDialog_launch()}
				</Button>
			{:else}
				<Button
					color="primary"
					icon="mdi:lan-connect"
					loading={activity === 'testing'}
					disabled={activity !== null}
					onclick={testConnection}
				>
					{m.dedicatedServerDialog_test()}
				</Button>
				<Button
					icon="mdi:cloud-upload"
					loading={activity === 'previewing' || activity === 'deploying'}
					disabled={activity !== null}
					onclick={deploy}
				>
					{m.dedicatedServerDialog_deploy()}
				</Button>
			{/if}
		</div>
	{/if}
</Dialog>

<DedicatedServerSettingsDialog bind:open={settingsDialogOpen} onsaved={settingsSaved} />
<DeploymentPreviewDialog bind:this={deploymentPreviewDialog} />
<DeploymentResultDialog bind:this={deploymentResultDialog} />
