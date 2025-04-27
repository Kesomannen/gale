<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { activeProfile, login, logout, refreshProfiles, user } from '$lib/stores';
	import { pushInfoToast } from '$lib/toast';
	import { discordAvatarUrl } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { Button } from 'bits-ui';

	type State = 'off' | 'synced' | 'outdated';

	let popupOpen = false;
	let loginLoading = false;
	let loading = false;

	$: syncInfo = $activeProfile?.sync ?? null;
	$: isOwner = syncInfo?.owner.discordId == $user?.discordId;
	$: state = (
		syncInfo === null
			? 'off'
			: new Date(syncInfo.updatedAt) > new Date(syncInfo.syncedAt)
				? 'outdated'
				: 'synced'
	) as State;

	$: style = {
		off: {
			icon: 'mdi:cloud-off',
			label: 'Sync off',
			classes: 'bg-primary-800 text-primary-400 hover:bg-primary-700'
		},
		synced: {
			icon: 'mdi:cloud-check-variant',
			label: 'Up to date',
			classes: 'bg-primary-800 text-accent-400 hover:bg-primary-700'
		},
		outdated: {
			icon: 'mdi:cloud-refresh-variant',
			label: 'Outdated',
			classes: 'bg-primary-800 text-yellow-400 hover:bg-primary-700'
		}
	}[state];

	async function onLoginClicked() {
		loginLoading = true;
		try {
			if ($user === null) {
				await login();
			} else {
				await logout();
			}
		} finally {
			loginLoading = false;
		}
	}

	async function connect() {
		await wrapCommand('create_sync_profile', 'Created synced profile.');
	}

	async function push() {
		await wrapCommand('push_sync_profile', 'Pushed update to synced profile.');
		popupOpen = false;
	}

	async function pull() {
		await wrapCommand('pull_sync_profile', 'Pulled changes from synced profile.');
		popupOpen = false;
	}

	async function fetch() {
		await wrapCommand('fetch_sync_profile');
	}

	async function disconnect() {
		let del = isOwner && (await ask('Do you also want to delete the profile from the database?'));

		await wrapCommand('disconnect_sync_profile', 'Disconnected synced profile.', { delete: del });
		popupOpen = false;
	}

	async function wrapCommand(command: string, message?: string, args?: any) {
		loading = true;
		try {
			await invokeCommand(command, args);
			await refreshProfiles();

			if (message) {
				pushInfoToast({ message });
			}
		} finally {
			loading = false;
		}
	}
</script>

<Button.Root
	class="{style.classes} mx-2 my-auto flex items-center gap-1 rounded-md px-2.5 py-1 text-sm"
	on:click={() => (popupOpen = true)}
>
	<Icon icon={style.icon} />
	<div>{style.label}</div>
</Button.Root>

<Popup bind:open={popupOpen} title="Profile sync">
	{#if syncInfo !== null}
		{#if !isOwner}
			<div class="text-primary-300 mt-2 flex items-center">
				<img
					src={discordAvatarUrl(syncInfo.owner)}
					alt=""
					class="mr-2 size-10 rounded-full shadow-lg"
				/>
				<div>
					Owned by {syncInfo.owner.displayName}
				</div>
			</div>
		{/if}

		<div class="mt-2 flex items-center gap-1">
			<Tooltip text="Copy to clipboard">
				<Button.Root
					class="rounded-md bg-slate-900 px-4 py-1 font-mono text-lg text-slate-300"
					on:click={async () => {
						await writeText(syncInfo.id);
						pushInfoToast({
							message: 'Copied profile code to clipboard.'
						});
					}}
				>
					{syncInfo.id}
				</Button.Root>
			</Tooltip>
		</div>

		<div class="mt-2 flex flex-wrap items-center gap-2">
			{#if state === 'outdated'}
				<BigButton on:click={pull} disabled={loading}>
					<Icon icon="mdi:cloud-download" class="mr-2 text-lg" />
					Pull update
				</BigButton>
			{/if}

			{#if isOwner}
				<BigButton on:click={push} disabled={loading || $user === null} color="accent">
					<Icon icon="mdi:cloud-upload" class="mr-2 text-lg" />
					Push update
				</BigButton>
			{/if}

			<BigButton on:click={fetch} disabled={loading} color="primary">
				<Icon icon="mdi:cloud-refresh" class="mr-2 text-lg" />
				Refresh
			</BigButton>

			<BigButton on:click={disconnect} disabled={loading} color="primary">
				<Icon icon="mdi:cloud-remove" class="mr-2 text-lg" />
				Disconnect
			</BigButton>
		</div>
	{:else if $user !== null}
		<BigButton on:click={connect} disabled={loading} color="accent" class="mt-2">
			<Icon icon="mdi:cloud-plus" class="mr-2 text-lg" />
			Connect
		</BigButton>
	{:else}
		<div class="text-primary-300">You must be logged in to create a synced profile.</div>
	{/if}

	<div class="mt-4 flex items-center text-slate-300">
		{#if $user !== null}
			<img src={discordAvatarUrl($user)} alt="" class="mr-2 size-10 rounded-full shadow-lg" />
		{/if}

		<BigButton on:click={onLoginClicked} disabled={loginLoading} color="primary">
			<Icon
				icon={loginLoading ? 'mdi:loading' : $user === null ? 'ic:baseline-discord' : 'mdi:logout'}
				class="mr-2 {loginLoading && 'animate-spin'}"
			/>

			{#if $user === null}
				Login with Discord
			{:else}
				Log out
			{/if}
		</BigButton>
	</div>
</Popup>
