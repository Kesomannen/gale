<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import SmallHeading from '$lib/prefs/SmallHeading.svelte';
	import { activeProfile, login, logout, refreshProfiles, user } from '$lib/stores';
	import { pushInfoToast } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Button } from 'bits-ui';

	let loginLoading = false;
	let loading = false;

	let newId = '';

	$: syncData = $activeProfile?.sync ?? null;
	$: isOwner = syncData?.ownerId == $user?.id;
	$: outOfDate =
		syncData !== null && new Date(syncData.lastUpdatedByOwner) > new Date(syncData.lastSynced);

	async function onLoginClicked() {
		loginLoading = true;
		try {
			if ($user === null) {
				await login('discord');
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
		if (outOfDate) {
			let confirmed = await confirm(
				'Are you sure you want to force push? This will override any changes that were made.'
			);
			if (!confirmed) return;
		}

		await wrapCommand('push_sync_profile', 'Pushed update to synced profile.');
	}

	async function pull() {
		await wrapCommand('pull_sync_profile', 'Pulled changes from synced profile.');
	}

	async function fetch() {
		await wrapCommand('fetch_sync_profile');
	}

	async function clone() {
		await wrapCommand('clone_sync_profile', 'Cloned synced profile.', { id: newId });
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

<div class="mx-auto flex w-full max-w-4xl flex-col overflow-y-auto px-6 pt-2 pb-6">
	<div class="mt-2 rounded-lg bg-slate-700 px-6 py-4">
		<div
			class="flex items-center gap-2 {syncData === null
				? 'text-slate-100'
				: outOfDate
					? 'text-yellow-400'
					: 'text-accent-400'}"
		>
			<Icon
				icon={loading
					? 'mdi:loading'
					: syncData === null
						? 'mdi:cloud-cancel'
						: outOfDate
							? 'mdi:cloud-refresh'
							: 'mdi:cloud-check'}
				class="text-3xl {loading && 'animate-spin'}"
			/>

			<div class="text-2xl font-bold">
				{#if syncData === null}
					{#if loading}
						Connecting...
					{:else}
						Profile is not connected
					{/if}
				{:else if loading}
					Syncing...
				{:else if outOfDate}
					Profile is out of date
				{:else}
					Profile is up to date
				{/if}
			</div>

			{#if syncData === null}
				{#if $user === null}
					<div class="ml-auto text-slate-300">
						You must to be logged in to create a synced profile.
					</div>
				{:else}
					<BigButton class="ml-auto" on:click={connect} disabled={loading}>
						<Icon icon="mdi:cloud-upload" class="mr-2 text-lg" />
						Connect
					</BigButton>
				{/if}
			{/if}
		</div>

		{#if syncData !== null}
			<div class="mt-2 flex items-center gap-1">
				<Tooltip text="Copy id to clipboard">
					<Button.Root
						class="rounded bg-slate-800 px-3 py-0.5 font-mono text-lg text-slate-300"
						on:click={async () => {
							await writeText(syncData.id);
							pushInfoToast({
								message: 'Copied profile id to clipboard.'
							});
						}}
					>
						{syncData.id}
					</Button.Root>
				</Tooltip>
			</div>

			<div class="mt-2 text-slate-300">
				<div>
					{#if isOwner}
						You are the owner of this profile
					{:else}
						You are <b>not</b> the owner of this profile
					{/if}
				</div>

				<div>
					Last synced: {new Date(syncData.lastSynced).toLocaleString()}
				</div>

				<div>
					Last updated: {new Date(syncData.lastUpdatedByOwner).toLocaleString()}
				</div>
			</div>

			<div class="mt-2">
				{#if outOfDate}
					<BigButton on:click={pull} disabled={loading || !outOfDate}>
						<Icon icon="mdi:cloud-download" class="mr-2 text-lg" />
						Pull update
					</BigButton>
				{/if}

				{#if $user === null || isOwner}
					<BigButton
						on:click={push}
						disabled={loading || $user === null}
						color={outOfDate ? 'slate' : 'accent'}
					>
						<Icon icon="mdi:cloud-upload" class="mr-2 text-lg" />
						Push update
					</BigButton>
				{/if}

				<BigButton on:click={fetch} disabled={loading} color="slate">
					<Icon icon="mdi:cloud-sync" class="mr-2 text-lg" />
					Refresh
				</BigButton>
			</div>
		{/if}
	</div>

	<div class="mt-2">
		<SmallHeading>Clone new profile</SmallHeading>
		<InputField bind:value={newId} placeholder="Enter profile sync id..." />
		<BigButton on:click={clone} disabled={loading}>Clone</BigButton>
	</div>

	<div class="mt-4 text-slate-300">
		<div class="flex items-center">
			{#if $user !== null}
				<img src={$user.avatarUrl} alt="" class="mr-2 size-8 rounded-full shadow-lg" />
				Logged in as {$user.displayName ?? $user.name}
			{/if}
		</div>

		<BigButton on:click={onLoginClicked} disabled={loginLoading} color="slate" class="mt-2">
			<Icon
				icon={loginLoading ? 'mdi:loading' : $user === null ? 'mdi:discord' : 'mdi:logout'}
				class="mr-2 {loginLoading && 'animate-spin'}"
			/>

			{#if $user === null}
				Login with Discord
			{:else}
				Log out
			{/if}
		</BigButton>
	</div>
</div>
