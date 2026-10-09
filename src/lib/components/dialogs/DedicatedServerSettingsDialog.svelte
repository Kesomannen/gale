<script lang="ts">
	import LocalServerSettings from './LocalServerSettings.svelte';
	import RemoteServerSettings from './RemoteServerSettings.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import games from '$lib/state/game.svelte';
	import { pushInfoToast, pushToast } from '$lib/toast';
	import type { ProfileServerSettings } from '$lib/types';
	import { Tabs } from 'bits-ui';

	const MAX_PORT = 65535;
	const DEFAULT_VALHEIM_PORT = 2456;
	const DEFAULT_SFTP_PORT = 22;

	type Props = {
		open?: boolean;
		onsaved?: (settings: ProfileServerSettings) => void;
	};

	let { open = $bindable(false), onsaved }: Props = $props();
	let settings = $state<ProfileServerSettings>(createSettings());
	let gamePassword = $state('');
	let remoteCredential = $state('');
	let activity = $state<'loading' | 'saving' | null>(null);
	let initialized = $state(false);

	$effect(() => {
		if (!open) {
			initialized = false;
			return;
		}
		if (!initialized) void load();
	});

	function createSettings(): ProfileServerSettings {
		const game = games.active;
		const dedicated = game?.dedicatedServer;

		return {
			location: 'local',
			local: {
				type: 'valheim',
				serverName: m.dedicatedServerSettings_defaultName({
					game: game?.name ?? m.unknown()
				}),
				world: m.dedicatedServerSettings_defaultWorld(),
				port: dedicated?.defaultPort ?? DEFAULT_VALHEIM_PORT,
				publicServer: true,
				crossplay: false,
				extraArgs: ''
			},
			remote: {
				protocol: 'sftp',
				host: '',
				port: DEFAULT_SFTP_PORT,
				username: '',
				serverDirectory: '/',
				authentication: 'password',
				privateKeyPath: '',
				trustedHostKey: null,
				trustedInvalidCertificateHost: null
			}
		};
	}

	async function load() {
		initialized = true;
		activity = 'loading';
		try {
			gamePassword = '';
			remoteCredential = '';
			const stored = await api.profile.server.getSettings();
			settings = stored ?? createSettings();
		} finally {
			activity = null;
		}
	}

	function validPort(port: number, label: string) {
		if (!Number.isInteger(port) || port < 1 || port > MAX_PORT) {
			pushToast({
				type: 'error',
				message: m.dedicatedServerDialog_portError({ label })
			});
			return false;
		}
		return true;
	}

	async function save() {
		const portIsValid =
			settings.location === 'local'
				? validPort(settings.local.port, m.dedicatedServerDialog_serverPort())
				: validPort(
						settings.remote.port,
						settings.remote.protocol === 'sftp'
							? m.dedicatedServerDialog_sshPort()
							: m.dedicatedServerDialog_ftpPort()
					);
		if (!portIsValid) {
			return;
		}

		activity = 'saving';
		try {
			settings.local.serverName = settings.local.serverName.trim();
			settings.local.world = settings.local.world.trim();
			settings.local.extraArgs = settings.local.extraArgs.trim();
			settings.remote.host = settings.remote.host.trim();
			settings.remote.username = settings.remote.username.trim();
			settings.remote.serverDirectory = settings.remote.serverDirectory.trim() || '/';
			settings.remote.privateKeyPath = settings.remote.privateKeyPath.trim();

			await api.profile.server.setSettings(settings, gamePassword, remoteCredential);
			gamePassword = '';
			remoteCredential = '';
			pushInfoToast({ message: m.dedicatedServerDialog_saved() });
			onsaved?.(settings);
			open = false;
		} finally {
			activity = null;
		}
	}
</script>

<Dialog title={m.dedicatedServerSettings_title()} bind:open large canClose={activity === null}>
	<p class="text-primary-600 dark:text-primary-300 mt-1">
		{m.dedicatedServerSettings_content()}
	</p>

	{#if activity === 'loading'}
		<div class="text-primary-500 dark:text-primary-400 flex justify-center py-12 text-3xl">
			<Spinner />
		</div>
	{:else}
		<TabsMenu
			bind:value={settings.location}
			options={[
				{ value: 'local', label: m.dedicatedServerDialog_locationLocal() },
				{ value: 'remote', label: m.dedicatedServerDialog_locationRemote() }
			]}
		>
			<Tabs.Content value="local">
				<LocalServerSettings bind:settings={settings.local} bind:password={gamePassword} />
			</Tabs.Content>

			<Tabs.Content value="remote">
				<RemoteServerSettings bind:settings={settings.remote} bind:credential={remoteCredential} />
			</Tabs.Content>
		</TabsMenu>
	{/if}

	<div class="mt-5 flex justify-end gap-2">
		<Button color="primary" disabled={activity !== null} onclick={() => (open = false)}>
			{m.dedicatedServerDialog_cancel()}
		</Button>
		<Button
			icon="mdi:content-save"
			loading={activity === 'saving'}
			disabled={activity !== null}
			onclick={save}
		>
			{m.dedicatedServerDialog_save()}
		</Button>
	</div>
</Dialog>
