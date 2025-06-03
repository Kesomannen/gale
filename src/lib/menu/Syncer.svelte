<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ListedSyncProfile } from '$lib/models';
	import { activeProfile, login, logout, refreshProfiles, user } from '$lib/stores';
	import { pushInfoToast } from '$lib/toast';
	import { discordAvatarUrl, timeSince } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { Button, DropdownMenu } from 'bits-ui';
	import OwnedSyncProfilesPopup from './OwnedSyncProfilesPopup.svelte';
	import { dropTransition } from '$lib/transitions';

	type State = 'off' | 'synced' | 'outdated';

	let mainPopupOpen = false;
	let loginLoading = false;
	let loading = false;

	let profilesPopupOpen = false;
	let profiles: ListedSyncProfile[] = [];

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
			classes: 'text-primary-400'
		},
		synced: {
			icon: 'mdi:cloud-check-variant',
			label: 'Up to date',
			classes: 'text-accent-400'
		},
		outdated: {
			icon: 'mdi:cloud-refresh-variant',
			label: 'Outdated',
			classes: 'text-yellow-400'
		}
	}[state];

	const dropdownItems = [
		{
			icon: 'mdi:database-eye',
			label: 'Show owned profiles',
			onClick: showOwnedProfiles
		},
		{
			icon: 'mdi:logout',
			label: 'Sign out',
			onClick: onLoginClicked
		}
	];

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
		mainPopupOpen = false;
	}

	async function pull() {
		await wrapCommand('pull_sync_profile', 'Pulled changes from synced profile.');
		mainPopupOpen = false;
	}

	async function refresh() {
		await wrapCommand('fetch_sync_profile', 'Refresh synced profile status.');
	}

	async function disconnect() {
		let del = isOwner && (await ask('Do you also want to delete the profile from the database?'));

		await wrapCommand('disconnect_sync_profile', 'Disconnected synced profile.', { delete: del });
		mainPopupOpen = false;
	}

	async function showOwnedProfiles() {
		loading = true;
		try {
			profiles = await invokeCommand<ListedSyncProfile[]>('get_owned_sync_profiles');

			mainPopupOpen = false;
			profilesPopupOpen = true;
		} finally {
			loading = false;
		}
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
	class="{style.classes} bg-primary-800 hover:bg-primary-700 mx-2 my-auto flex items-center gap-1 rounded-md px-2.5 py-1 text-sm"
	on:click={() => (mainPopupOpen = true)}
>
	<Icon icon={style.icon} />
	<div>{style.label}</div>
</Button.Root>

<OwnedSyncProfilesPopup
	bind:open={profilesPopupOpen}
	{profiles}
	onClose={() => (mainPopupOpen = true)}
/>

<Popup bind:open={mainPopupOpen} title="Profile sync">
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
					class="rounded-md bg-primary-900 px-4 py-1 font-mono text-lg text-primary-300"
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

			<BigButton on:click={refresh} disabled={loading} color="primary">
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
	{/if}

	<div class="mt-4 flex items-center gap-1 text-primary-300">
		{#if $user === null}
			<BigButton on:click={onLoginClicked} disabled={loginLoading} color="primary">
				<Icon
					icon={loginLoading ? 'mdi:loading' : 'ic:baseline-discord'}
					class="mr-2 {loginLoading && 'animate-spin'}"
				/>

				Sign in with Discord
			</BigButton>
		{:else}
			<img src={discordAvatarUrl($user)} alt="" class="size-10 rounded-full shadow-lg" />

			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="bg-primary-800 hover:bg-primary-700 rounded-full p-1">
					<Icon class="text-2xl" icon="mdi:dots-vertical" />
				</DropdownMenu.Trigger>
				<DropdownMenu.Content
					class="border-primary-600 bg-primary-800 flex flex-col gap-0.5 rounded-lg border p-1 shadow-xl"
					side="bottom"
					{...dropTransition}
				>
					{#each dropdownItems as item}
						<DropdownMenu.Item class="menu-item context-menu-item pr-6" on:click={item.onClick}>
							<Icon icon={item.icon} class="mr-1.5 text-lg" />

							{item.label}
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/if}
	</div>

	<div
		class="text-primary-400 text-primary-400 hover:text-accent-400 mt-4 flex max-w-max items-center gap-1 text-sm hover:underline"
	>
		<Icon icon="mdi:help-circle" inline />

		<a target="_blank" href="https://github.com/Kesomannen/gale/wiki/Profile-sync/">What is this?</a
		>
	</div>
</Popup>
