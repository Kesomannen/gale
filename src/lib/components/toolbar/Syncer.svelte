<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import SyncAvatar from '$lib/components/ui/SyncAvatar.svelte';
	import * as api from '$lib/api';
	import type { ListedSyncProfile } from '$lib/types';
	import { pushInfoToast } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { DropdownMenu } from 'bits-ui';
	import OwnedSyncProfilesDialog from '../dialogs/OwnedSyncProfilesDialog.svelte';
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import profiles from '$lib/state/profile.svelte';
	import user from '$lib/state/user.svelte';
	import Link from '../ui/Link.svelte';
	import { PersistedState } from 'runed';

	type State = 'off' | 'synced' | 'outdated';

	let mainDialogOpen = $state(false);
	let loginLoading = $state(false);
	let loading = $state(false);

	let profilesDialogOpen = $state(false);
	let syncProfiles: ListedSyncProfile[] = $state([]);

	let syncInfo = $derived(profiles.active?.sync ?? null);
	let isOwner = $derived(syncInfo?.owner.discordId == user.value?.discordId);
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
			onclick: showOwnedProfiles
		},
		{
			icon: 'mdi:logout',
			label: 'Sign out',
			onclick: onLoginClicked
		}
	];

	const donationCloseDuration = 1000 * 60 * 60 * 24 * 7; // 1 week
	let donationClosedAt = new PersistedState<string | null>('donationClosedAt', null);

	let showDonation = $derived(
		syncInfo &&
			(true ||
				!donationClosedAt.current ||
				Date.now() - new Date(donationClosedAt.current).getTime() > donationCloseDuration)
	);

	async function onLoginClicked() {
		loginLoading = true;
		try {
			if (user.value === null) {
				let userInfo = await user.login();
				pushInfoToast({
					message: `Signed in with Discord as ${userInfo.displayName}.`
				});
			} else {
				await user.logout();
			}
		} finally {
			loginLoading = false;
		}
	}

	async function connect() {
		await wrapApiCall(api.profile.sync.create, 'Created synced profile.');
	}

	async function push() {
		await wrapApiCall(api.profile.sync.push, 'Pushed update to synced profile.');
	}

	async function pull() {
		await wrapApiCall(api.profile.sync.pull, 'Pulled changes from synced profile.');
	}

	async function refresh() {
		await wrapApiCall(api.profile.sync.fetch, 'Refreshed synced profile status.');
	}

	async function disconnect() {
		let del = isOwner && (await ask('Do you also want to delete the profile from the database?'));

		await wrapApiCall(() => api.profile.sync.disconnect(del), 'Disconnected synced profile.');
		mainDialogOpen = false;
	}

	async function showOwnedProfiles() {
		loading = true;
		try {
			syncProfiles = await api.profile.sync.getOwned();

			mainDialogOpen = false;
			profilesDialogOpen = true;
		} finally {
			loading = false;
		}
	}

	async function wrapApiCall(call: () => Promise<any>, message?: string) {
		loading = true;
		try {
			await call();
			await profiles.refresh();

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
	onclick={() => (mainDialogOpen = true)}
>
	<Icon icon={style.icon} />
	<div>{style.label}</div>
</button>

<OwnedSyncProfilesDialog
	bind:open={profilesDialogOpen}
	profiles={syncProfiles}
	onClose={() => (mainDialogOpen = true)}
/>

<Dialog bind:open={mainDialogOpen} title="Profile sync">
	<div
		class={[
			!showDonation && 'hidden',
			'bg-primary-900 relative my-2 overflow-hidden rounded-md py-4 pr-4 pl-6'
		]}
	>
		<div class="bg-accent-600 absolute top-0 bottom-0 left-0 w-1"></div>

		<div
			class="text-lg font-semibold
			 text-white"
		>
			Profile sync is run on donations!
		</div>

		<div class="text-primary-300">
			If you like this feature, please consider supporting on <Link
				href="https://ko-fi.com/kesomannen">Kofi</Link
			>

			<Icon class="mb-1 inline" icon="mdi:heart" />.
		</div>

		<button
			class="text-primary-400 hover:text-accent-400 mt-2 flex items-center gap-1 text-sm hover:underline"
			onclick={() => {
				donationClosedAt.current = new Date().toISOString();
			}}
		>
			<Icon icon="mdi:close" />
			Remind me later
		</button>
	</div>

	{#if syncInfo}
		{#if !isOwner}
			<div class="text-primary-300 mt-2 flex items-center gap-2">
				<SyncAvatar user={syncInfo.owner} />
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
					disabled={user.value === null}
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
	{:else if user.value !== null}
		<Button onclick={connect} {loading} color="accent" class="mt-2" icon="mdi:cloud-plus">
			Connect
		</Button>
	{/if}

	<div class="text-primary-300 mt-4 flex items-center gap-1">
		{#if user.value === null}
			<Button
				onclick={onLoginClicked}
				loading={loginLoading}
				color="primary"
				icon="ic:baseline-discord"
			>
				Sign in with Discord
			</Button>
		{:else}
			<SyncAvatar user={user.value} />

			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="bg-primary-800 hover:bg-primary-700 rounded-full p-1">
					<Icon class="text-2xl" icon="mdi:dots-vertical" />
				</DropdownMenu.Trigger>
				<ContextMenuContent type="dropdown" style="dark" items={dropdownItems} />
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
</Dialog>
