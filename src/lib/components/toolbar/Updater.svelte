<script module lang="ts">
	import { check, type Update } from '@tauri-apps/plugin-updater';
	import { writable } from 'svelte/store';

	export let nextUpdate = writable<Update | null>(null);
	export let isChecking = writable(false);

	export async function refreshUpdate() {
		isChecking.set(true);
		let update = await check();
		isChecking.set(false);

		if (update === null || !update.available) return;
		nextUpdate.set(update);
	}
</script>

<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Icon from '@iconify/svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { platform } from '@tauri-apps/plugin-os';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Dialog } from 'bits-ui';
	import { onMount } from 'svelte';
	import { pushToast } from '$lib/toast';

	let popupOpen = $state(false);
	let loading = $state(false);

	onMount(() => {
		refreshUpdate();
	});

	async function installUpdate() {
		if ($nextUpdate === null) return;

		try {
			await $nextUpdate.downloadAndInstall();
		} catch (error) {
			let message: string;
			if (typeof error === 'string') {
				message = error;
			} else if (error instanceof Error) {
				message = error.message;
			} else {
				message = 'Unknown error';
			}

			pushToast({
				type: 'error',
				name: 'Failed to update Gale',
				message
			});
		}

		$nextUpdate = null;
	}

	async function update() {
		popupOpen = false;
		loading = true;
		await installUpdate();
		loading = false;

		if (platform() !== 'windows') {
			// on other platforms installUpdate() doesn't relaunch the app itself
			await message('Gale will now restart in order to apply the update.');
			await relaunch();
		}
	}
</script>

{#if $nextUpdate !== null}
	<button
		class="bg-accent-700 enabled:hover:bg-accent-600 text-primary-100 my-auto mr-2 ml-auto flex items-center gap-1 rounded-md px-2.5 py-1 text-sm"
		disabled={loading}
		onclick={() => (popupOpen = true)}
	>
		{#if loading}
			<Icon icon="mdi:loading" class="animate-spin" />
		{:else}
			<Icon icon="mdi:arrow-up-circle" />
		{/if}
		<span class="text-sm">{loading ? 'Downloading update...' : 'Update available'}</span>
	</button>
{/if}

<ConfirmDialog title="App update available" bind:open={popupOpen}>
	<Dialog.Description class="text-primary-300">
		<p>
			{#if nextUpdate}
				Version {$nextUpdate?.version} of Gale is available - you have {$nextUpdate?.currentVersion}.
			{:else}
				There is an update available for Gale.
			{/if}

			The update will be downloaded in the background, then the app will restart to apply it.
		</p>
		<p class="mt-1">Would you like to install it?</p>
	</Dialog.Description>

	{#snippet buttons()}
		<Button color="accent" onclick={update}>Install</Button>
	{/snippet}
</ConfirmDialog>
