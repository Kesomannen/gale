<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
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
	import auth from '$lib/state/auth.svelte';
	import Link from '../ui/Link.svelte';
	import { PersistedState } from 'runed';
	import IconButton from '../ui/IconButton.svelte';
	import Spinner from '../ui/Spinner.svelte';
	import InfoBox from '../ui/InfoBox.svelte';
	import SyncDonationNotice from './SyncDonationNotice.svelte';

	type State = 'off' | 'synced' | 'outdated' | 'missing';

	let mainDialogOpen = $state(false);
	let loginLoading = $state(false);
	let loading = $state(false);

	let profilesDialogOpen = $state(false);
	let syncProfiles: ListedSyncProfile[] = $state([]);

	let syncInfo = $derived(profiles.active?.sync ?? null);
	let isOwner = $derived(syncInfo?.owner.discordId == auth.user?.discordId);
	let syncState = $derived(
		(syncInfo === null
			? 'off'
			: syncInfo.missing
				? 'missing'
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
			},
			missing: {
				icon: 'mdi:cloud-alert',
				label: 'Sync error',
				classes: 'text-red-500 font-medium'
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

	const copyItems = [
		{
			icon: 'mdi:clipboard-text',
			label: 'Copy profile code',
			onclick: copyCode
		},
		{
			icon: 'mdi:link',
			label: 'Copy import link',
			onclick: copyLink
		}
	];

	async function onLoginClicked() {
		loginLoading = true;
		try {
			if (auth.user === null) {
				let userInfo = await auth.login();
				pushInfoToast({
					message: `Signed in with Discord as ${userInfo.displayName}.`
				});
			} else {
				await auth.logout();
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
		let deleteFromRemote =
			isOwner &&
			syncState !== 'missing' &&
			(await ask('Do you also want to delete the profile from the database?'));

		await wrapApiCall(
			() => api.profile.sync.disconnect(deleteFromRemote),
			'Disconnected synced profile.'
		);
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
			if (message) {
				pushInfoToast({ message });
			}
		} finally {
			loading = false;
		}
	}

	async function copyCode() {
		if (!syncInfo) return;

		await writeText(syncInfo.id);
		pushInfoToast({
			message: 'Copied profile code to clipboard.'
		});
	}

	async function copyLink() {
		if (!syncInfo) return;

		let url = `https://gale.kesomannen.com/api/desktop/profile/sync/clone/${syncInfo.id}`;
		await writeText(url);
		pushInfoToast({
			message: 'Copied profile import link to clipboard.'
		});
	}
</script>

<button
	class="{style.classes} bg-primary-800 hover:bg-primary-700 mx-2 my-auto flex items-center gap-1.5 rounded-md px-2.5 py-1 text-sm"
	onclick={() => (mainDialogOpen = true)}
>
	{#if loading}
		<Spinner />
	{:else}
		<Icon icon={style.icon} />
	{/if}

	<div>{style.label}</div>
</button>

<OwnedSyncProfilesDialog
	bind:open={profilesDialogOpen}
	profiles={syncProfiles}
	onClose={() => (mainDialogOpen = true)}
/>

<Dialog bind:open={mainDialogOpen} title="Profile sync">
	<SyncDonationNotice show={syncInfo !== null} />

	{#if syncInfo}
		{#if syncState !== 'missing'}
			{#if !isOwner}
				<div class="text-primary-300 mt-2 flex items-center gap-2">
					<SyncAvatar user={syncInfo.owner} />
					<div>
						Owned by {syncInfo.owner.displayName}
					</div>
				</div>
			{/if}

			<div class="mt-2 flex items-center gap-2">
				<button
					class="bg-primary-900 text-primary-300 rounded-md px-4 py-1 font-mono text-lg"
					onclick={copyCode}
				>
					{syncInfo.id}
				</button>

				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<IconButton icon="mdi:content-copy" label="Copy to clipboard" />
					</DropdownMenu.Trigger>
					<ContextMenuContent type="dropdown" style="dark" items={copyItems} />
				</DropdownMenu.Root>
			</div>
		{:else}
			<InfoBox type="error">
				This profile has been deleted and can no longer receive updates or be imported.
			</InfoBox>
		{/if}

		<div class="mt-2 flex flex-wrap items-center gap-2">
			{#if syncState !== 'missing'}
				{#if syncState === 'outdated'}
					<Button onclick={pull} {loading} icon="mdi:cloud-download">Pull update</Button>
				{/if}

				{#if isOwner}
					<Button
						onclick={push}
						{loading}
						disabled={auth.user === null}
						color="accent"
						icon="mdi:cloud-upload"
					>
						Push update
					</Button>
				{/if}

				<Button onclick={refresh} {loading} color="primary" icon="mdi:cloud-refresh">Refresh</Button
				>
			{/if}

			<Button
				onclick={disconnect}
				{loading}
				color={syncState === 'missing' ? 'accent' : 'primary'}
				icon="mdi:cloud-remove"
			>
				Disconnect
			</Button>
		</div>
	{:else if auth.user !== null}
		<Button onclick={connect} {loading} color="accent" class="mt-2" icon="mdi:cloud-plus">
			Connect
		</Button>
	{/if}

	<div class="text-primary-300 mt-4 flex items-center gap-1">
		{#if auth.user === null}
			<Button
				onclick={onLoginClicked}
				loading={loginLoading}
				color="primary"
				icon="ic:baseline-discord"
			>
				Sign in with Discord
			</Button>
		{:else}
			<SyncAvatar user={auth.user} />

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
