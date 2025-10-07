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
	import IconButton from '../ui/IconButton.svelte';
	import InfoBox from '../ui/InfoBox.svelte';
	import SyncDonationNotice from './SyncDonationNotice.svelte';
	import { m } from '$lib/paraglide/messages';

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
				label: m.syncer_style_off(),
				classes: 'text-primary-400'
			},
			synced: {
				icon: 'mdi:cloud-check-variant',
				label: m.syncer_style_synced(),
				classes: 'text-accent-400'
			},
			outdated: {
				icon: 'mdi:cloud-refresh-variant',
				label: m.syncer_style_outdated(),
				classes: 'text-yellow-400'
			},
			missing: {
				icon: 'mdi:cloud-alert',
				label: m.syncer_style_missing(),
				classes: 'text-red-500 font-semibold'
			}
		}[syncState]
	);

	const dropdownItems = [
		{
			icon: 'mdi:database-eye',
			label: m.syncer_dropdownItems_showOwnedProfiles(),
			onclick: showOwnedProfiles
		},
		{
			icon: 'mdi:logout',
			label: m.syncer_dropdownItems_login(),
			onclick: onLoginClicked
		}
	];

	const copyItems = [
		{
			icon: 'mdi:clipboard-text',
			label: m.syncer_copyItems_copyCode(),
			onclick: copyCode
		},
		{
			icon: 'mdi:link',
			label: m.syncer_copyItems_copyLink(),
			onclick: copyLink
		}
	];

	async function onLoginClicked() {
		loginLoading = true;
		try {
			if (auth.user === null) {
				let userInfo = await auth.login();
				pushInfoToast({
					message: m.syncer_onLoginClicked_message({name : userInfo.displayName})
				});
			} else {
				await auth.logout();
			}
		} finally {
			loginLoading = false;
		}
	}

	async function connect() {
		await wrapApiCall(api.profile.sync.create, m.syncer_connect_message());
	}

	async function push() {
		await wrapApiCall(api.profile.sync.push, m.syncer_push_message());
	}

	async function pull() {
		await wrapApiCall(api.profile.sync.pull, m.syncer_pull_message());
	}

	async function refresh() {
		await wrapApiCall(api.profile.sync.fetch, m.syncer_refresh_message());
	}

	async function disconnect() {
		let deleteFromRemote =
			isOwner &&
			syncState !== 'missing' &&
			(await ask(m.syncer_disconnect_ask()));

		await wrapApiCall(
			() => api.profile.sync.disconnect(deleteFromRemote),
			m.syncer_disconnect_message()
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
			message: m.syncer_copyCode_message()
		});
	}

	async function copyLink() {
		if (!syncInfo) return;

		let url = `https://gale.kesomannen.com/api/desktop/profile/sync/clone/${syncInfo.id}`;
		await writeText(url);
		pushInfoToast({
			message: m.syncer_copyLink_message()
		});
	}
</script>

<button
	class={[
		style.classes,
		'bg-primary-800 hover:bg-primary-700 mx-2 my-auto flex items-center gap-1.5 rounded-md px-2.5 py-1 text-sm'
	]}
	onclick={() => (mainDialogOpen = true)}
>
	<Icon icon={style.icon} />

	<div class="truncate">{style.label}</div>
</button>

<OwnedSyncProfilesDialog
	bind:open={profilesDialogOpen}
	profiles={syncProfiles}
	onClose={() => (mainDialogOpen = true)}
/>

<Dialog bind:open={mainDialogOpen} title={m.syncer_title()}>
	<SyncDonationNotice show={syncInfo !== null} />

	{#if syncInfo}
		{#if syncState !== 'missing'}
			{#if !isOwner}
				<div class="text-primary-300 mt-2 flex items-center gap-2">
					<SyncAvatar user={syncInfo.owner} />
					<div>
						{m.syncer_content_1()}{syncInfo.owner.displayName}
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
						<IconButton icon="mdi:content-copy" label={m.syncer_button_copyConetnt()} />
					</DropdownMenu.Trigger>
					<ContextMenuContent type="dropdown" style="dark" items={copyItems} />
				</DropdownMenu.Root>
			</div>
		{:else}
			<InfoBox type="error">
				{m.syncer_content_2()}
			</InfoBox>
		{/if}

		<div class="mt-2 flex flex-wrap items-center gap-2">
			{#if syncState !== 'missing'}
				{#if syncState === 'outdated'}
					<Button onclick={pull} {loading} icon="mdi:cloud-download">{m.syncer_button_pull()}</Button>
				{/if}

				{#if isOwner}
					<Button
						onclick={push}
						{loading}
						disabled={auth.user === null}
						color="accent"
						icon="mdi:cloud-upload"
					>
						{m.syncer_button_push()}
					</Button>
				{/if}

				<Button onclick={refresh} {loading} color="primary" icon="mdi:cloud-refresh">{m.syncer_button_refresh()}</Button>
			{/if}

			<Button
				onclick={disconnect}
				{loading}
				color={syncState === 'missing' ? 'accent' : 'primary'}
				icon="mdi:cloud-remove"
			>
				{m.syncer_button_disconnect()}
			</Button>
		</div>
	{:else if auth.user !== null}
		<Button onclick={connect} {loading} color="accent" class="mt-2" icon="mdi:cloud-plus">
			{m.syncer_button_connect()}
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
				{m.syncer_button_login()}
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

	<div class="text-primary-400 hover:text-accent-400 mt-4 flex max-w-max items-center gap-1 text-sm hover:underline">
		<Icon icon="mdi:help-circle" inline />

		<a target="_blank" href="https://github.com/Kesomannen/gale/wiki/Profile-sync/">{m.syncer_content_help()}</a>
	</div>
</Dialog>
