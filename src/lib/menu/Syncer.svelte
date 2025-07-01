<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invoke } from '$lib/invoke';
	import type { ListedSyncProfile } from '$lib/types';
	import { activeProfile, login, logout, refreshProfiles, user } from '$lib/stores.svelte';
	import { pushInfoToast } from '$lib/toast';
	import { discordAvatarUrl } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { DropdownMenu } from 'bits-ui';
	import OwnedSyncProfilesPopup from './OwnedSyncProfilesPopup.svelte';

	type State = 'off' | 'synced' | 'outdated';

	let mainPopupOpen = $state(false);
	let loginLoading = $state(false);
	let loading = $state(false);

	let profilesPopupOpen = $state(false);
	let profiles: ListedSyncProfile[] = $state([]);

	let syncInfo = $derived($activeProfile?.sync ?? null);
	let isOwner = $derived(syncInfo?.owner.discordId == $user?.discordId);
	let syncState = $derived(
		(syncInfo === null
			? 'off'
			: new Date(syncInfo.updatedAt) > new Date(syncInfo.syncedAt)
				? 'outdated'
				: 'synced') as State
	);

	let style = $derived(
		{
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
		}[syncState]
	);

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
			profiles = await invoke<ListedSyncProfile[]>('get_owned_sync_profiles');

			mainPopupOpen = false;
			profilesPopupOpen = true;
		} finally {
			loading = false;
		}
	}

	async function wrapCommand(command: string, message?: string, args?: any) {
		loading = true;
		try {
			await invoke(command, args);
			await refreshProfiles();

			if (message) {
				pushInfoToast({ message });
			}
		} finally {
			loading = false;
		}
	}
</script>

<button
	class="{style.classes} bg-primary-800 hover:bg-primary-700 mx-2 my-auto flex items-center gap-1 rounded-md px-2.5 py-1 text-sm"
	onclick={() => (mainPopupOpen = true)}
>
	<Icon icon={style.icon} />
	<div>{style.label}</div>
</button>

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
				<button
					class="bg-primary-900 text-primary-300 rounded-md px-4 py-1 font-mono text-lg"
					onclick={async () => {
						await writeText(syncInfo.id);
						pushInfoToast({
							message: 'Copied profile code to clipboard.'
						});
					}}
				>
					{syncInfo.id}
				</button>
			</Tooltip>
		</div>

		<div class="mt-2 flex flex-wrap items-center gap-2">
			{#if syncState === 'outdated'}
				<Button onclick={pull} disabled={loading} icon="mdi:cloud-download">Pull update</Button>
			{/if}

			{#if isOwner}
				<Button
					onclick={push}
					{loading}
					disabled={$user === null}
					color="accent"
					icon="mdi:cloud-upload"
				>
					Push update
				</Button>
			{/if}

			<Button onclick={refresh} {loading} color="primary" icon="mdi:cloud-refresh">Refresh</Button>

			<Button onclick={disconnect} {loading} color="primary" icon="mdi:cloud-remove">
				Disconnect
			</Button>
		</div>
	{:else if $user !== null}
		<Button onclick={connect} {loading} color="accent" class="mt-2" icon="mdi:cloud-plus">
			Connect
		</Button>
	{/if}

	<div class="text-primary-300 mt-4 flex items-center gap-1">
		{#if $user === null}
			<Button
				onclick={onLoginClicked}
				loading={loginLoading}
				color="primary"
				icon="ic:baseline-discord"
			>
				Sign in with Discord
			</Button>
		{:else}
			<img src={discordAvatarUrl($user)} alt="" class="size-10 rounded-full shadow-lg" />

			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="bg-primary-800 hover:bg-primary-700 rounded-full p-1">
					<Icon class="text-2xl" icon="mdi:dots-vertical" />
				</DropdownMenu.Trigger>
				<DropdownMenu.Content
					class="border-primary-600 bg-primary-800 flex flex-col gap-0.5 rounded-lg border p-1 shadow-xl"
					side="bottom"
				>
					{#each dropdownItems as item}
						<DropdownMenu.Item class="menu-item context-menu-item pr-6" onclick={item.onClick}>
							<Icon icon={item.icon} class="mr-1.5 text-lg" />

							{item.label}
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/if}
	</div>

	<div
		class="text-primary-400 hover:text-accent-400 mt-4 flex max-w-max items-center gap-1 text-sm hover:underline"
	>
		<Icon icon="mdi:help-circle" inline />

		<a target="_blank" href="https://github.com/Kesomannen/gale/wiki/Profile-sync/">What is this?</a
		>
	</div>
</Popup>
