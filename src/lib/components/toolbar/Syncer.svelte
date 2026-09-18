<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import SyncAvatar from '$lib/components/ui/SyncAvatar.svelte';
	import * as api from '$lib/api';
	import type { ListedSyncProfile, SyncConfigReviewItem, SyncConfigReviewState } from '$lib/types';
	import { pushInfoToast } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { DropdownMenu } from 'bits-ui';
	import OwnedSyncProfilesDialog from '../dialogs/OwnedSyncProfilesDialog.svelte';
	import SyncPublishDialog from '../dialogs/SyncPublishDialog.svelte';
	import SyncConfigReviewDialog from '../dialogs/SyncConfigReviewDialog.svelte';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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

	let publishDialogOpen = $state(false);
	let reviewDialogOpen = $state(false);
	// captured when a dialog opens so its commands stay pinned to the profile
	// the user was looking at, even if the active profile changes mid-operation
	let syncDialogProfileId: number | null = $state(null);
	let reviewMode: 'pending' | 'declined' | 'policies' = $state('pending');
	let reviewState: SyncConfigReviewState = $state({ pending: [], declined: [], policies: [] });

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
				classes: 'text-primary-500 dark:text-primary-400'
			},
			synced: {
				icon: 'mdi:cloud-check-variant',
				label: m.syncer_style_synced(),
				classes: 'text-accent-600 dark:text-accent-500'
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
					message: m.syncer_onLoginClicked_message({ name: userInfo.displayName })
				});
			} else {
				await auth.logout();
			}
		} finally {
			loginLoading = false;
		}
	}

	async function connect() {
		const profileId = profiles.active?.id;
		if (profileId === undefined) return;
		await wrapApiCall(() => api.profile.sync.create(profileId), m.syncer_connect_message());
	}

	async function pull() {
		let key = activeKey();
		const profileId = profiles.active?.id;
		if (profileId === undefined) return;
		loading = true;
		try {
			await api.profile.sync.pull(profileId);
			await refreshPending(key);
			pushInfoToast({ message: m.syncer_pull_message() });
		} finally {
			loading = false;
		}
	}

	function activeKey(): string | undefined {
		let active = profiles.active;
		return active?.sync?.id === undefined ? undefined : `${active.id}:${active.sync.id}`;
	}

	async function refreshPending(key: string | undefined = activeKey()) {
		if (key === undefined) {
			reviewState = { pending: [], declined: [], policies: [] };
			return;
		}

		const profileId = profiles.active?.id;
		if (profileId === undefined) return;

		try {
			let updates = await api.profile.sync.getPendingConfig(profileId);
			if (activeKey() === key) {
				reviewState = updates;
			}
		} catch {}
	}

	let lastSyncKey: string | undefined;

	$effect(() => {
		let key = activeKey();
		if (key === lastSyncKey) return;
		lastSyncKey = key;
		reviewDialogOpen = false;
		publishDialogOpen = false;
		syncDialogProfileId = null;
		reviewMode = 'pending';
		reviewState = { pending: [], declined: [], policies: [] };

		if (key === undefined) {
			return;
		}

		refreshPending(key);
	});

	onMount(() => {
		let unlistenPending: UnlistenFn | null = null;
		let unlistenReview: UnlistenFn | null = null;

		listen<{ profileId: number; pending: SyncConfigReviewItem[] }>('sync_config_pending', (evt) => {
			if (evt.payload.profileId !== profiles.active?.id) return;
			pushInfoToast({
				message: m.syncer_pendingConfigToast({ count: evt.payload.pending.length })
			});
		}).then((callback) => (unlistenPending = callback));

		listen<{ profileId: number }>('sync_config_review_changed', (evt) => {
			if (evt.payload.profileId !== profiles.active?.id) return;
			refreshPending();
		}).then((callback) => (unlistenReview = callback));

		return () => {
			unlistenPending?.();
			unlistenReview?.();
		};
	});

	async function refresh() {
		const profileId = profiles.active?.id;
		if (profileId === undefined) return;
		await wrapApiCall(() => api.profile.sync.fetch(profileId), m.syncer_refresh_message());
	}

	async function disconnect() {
		// capture before the confirmation dialog so the operation stays pinned
		// to the profile that was active when it was clicked
		const profileId = profiles.active?.id;
		if (profileId === undefined) return;

		let deleteFromRemote =
			isOwner && syncState !== 'missing' && (await ask(m.syncer_disconnect_ask()));

		await wrapApiCall(
			() => api.profile.sync.disconnect(deleteFromRemote, profileId),
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
		'dark:bg-primary-800 dark:hover:bg-primary-700 bg-primary-200 hover:bg-primary-300 mx-2 my-auto flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1 text-sm'
	]}
	onclick={() => {
		mainDialogOpen = true;
		refreshPending();
	}}
>
	<Icon class="text-lg md:text-base" icon={style.icon} />

	<div class="hidden md:block">{style.label}</div>

	{#if reviewState.pending.length > 0}
		<span
			class="bg-accent-600 rounded-full px-1.5 py-0.5 text-xs leading-none font-medium text-white"
		>
			{reviewState.pending.length}
		</span>
	{/if}
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
				<div class="text-primary-600 dark:text-primary-300 mt-2 flex items-center gap-2">
					<SyncAvatar user={syncInfo.owner} />
					<div>
						{m.syncer_content_1()}{syncInfo.owner.displayName}
					</div>
				</div>
			{/if}

			<div class="mt-2 flex items-center gap-2">
				<button
					class="text-primary-600 dark:bg-primary-900 dark:text-primary-300 bg-primary-100 rounded-md px-4 py-1 font-mono text-lg"
					onclick={copyCode}
				>
					{syncInfo.id}
				</button>

				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<IconButton icon="mdi:content-copy" label={m.syncer_button_copyConetnt()} />
					</DropdownMenu.Trigger>
					<ContextMenuContent type="dropdown" items={copyItems} />
				</DropdownMenu.Root>
			</div>
		{:else}
			<InfoBox type="error">
				{m.syncer_content_2()}
			</InfoBox>
		{/if}

		<div class="mt-2 flex flex-wrap items-center gap-2">
			{#if reviewState.pending.length > 0}
				<Button
					onclick={() => {
						reviewMode = 'pending';
						syncDialogProfileId = profiles.active?.id ?? null;
						reviewDialogOpen = true;
					}}
					color="primary"
					icon="mdi:file-document-edit"
				>
					{m.syncer_button_reviewConfig({ count: reviewState.pending.length })}
				</Button>
			{/if}

			{#if reviewState.declined.length > 0}
				<Button
					onclick={() => {
						reviewMode = 'declined';
						syncDialogProfileId = profiles.active?.id ?? null;
						reviewDialogOpen = true;
					}}
					color="primary"
					icon="mdi:file-document-remove"
				>
					{m.syncer_button_declinedConfig({ count: reviewState.declined.length })}
				</Button>
			{/if}

			{#if reviewState.policies.length > 0}
				<Button
					onclick={() => {
						reviewMode = 'policies';
						syncDialogProfileId = profiles.active?.id ?? null;
						reviewDialogOpen = true;
					}}
					color="primary"
					icon="mdi:tune"
				>
					{m.syncer_button_configPolicies()}
				</Button>
			{/if}

			{#if syncState !== 'missing'}
				{#if syncState === 'outdated'}
					<Button onclick={pull} {loading} icon="mdi:cloud-download"
						>{m.syncer_button_pull()}</Button
					>
				{/if}

				{#if isOwner}
					<Button
						onclick={() => {
							syncDialogProfileId = profiles.active?.id ?? null;
							publishDialogOpen = true;
						}}
						{loading}
						disabled={auth.user === null}
						color="accent"
						icon="mdi:cloud-upload"
					>
						{m.syncer_button_push()}
					</Button>
				{/if}

				<Button onclick={refresh} {loading} color="primary" icon="mdi:cloud-refresh"
					>{m.syncer_button_refresh()}</Button
				>
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

	<div class="text-primary-600 dark:text-primary-300 mt-4 flex items-center gap-1">
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
				<DropdownMenu.Trigger
					class="dark:bg-primary-800 dark:hover:bg-primary-700 bg-primary-200 hover:bg-primary-300 rounded-full p-1"
				>
					<Icon class="text-2xl" icon="mdi:dots-vertical" />
				</DropdownMenu.Trigger>
				<ContextMenuContent type="dropdown" items={dropdownItems} />
			</DropdownMenu.Root>
		{/if}
	</div>

	<div
		class="text-primary-500 hover:text-accent-600 dark:text-primary-400 dark:hover:text-accent-400 mt-4 flex max-w-max items-center gap-1 text-sm hover:underline"
	>
		<Icon icon="mdi:help-circle" inline />

		<a target="_blank" href="https://github.com/Kesomannen/gale/wiki/Profile-sync/"
			>{m.syncer_content_help()}</a
		>
	</div>
</Dialog>

{#if syncDialogProfileId !== null}
	<SyncPublishDialog
		bind:open={publishDialogOpen}
		profileId={syncDialogProfileId}
		onPublished={() => refreshPending()}
	/>

	<SyncConfigReviewDialog
		bind:open={reviewDialogOpen}
		profileId={syncDialogProfileId}
		updates={reviewState}
		mode={reviewMode}
		onChanged={() => refreshPending()}
	/>
{/if}
