<script context="module" lang="ts">
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
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { pushError } from '$lib/invoke';
	import Icon from '@iconify/svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { platform } from '@tauri-apps/plugin-os';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Button, Dialog } from 'bits-ui';
	import { onMount } from 'svelte';

	let popupOpen = false;
	let loading = false;

	onMount(() => {
		refreshUpdate();
	});

	async function installUpdate() {
		if ($nextUpdate === null) return;

		try {
			await $nextUpdate.downloadAndInstall();
		} catch (e) {
			let message: string;
			if (typeof e === 'string') {
				message = e;
			} else if (e instanceof Error) {
				message = e.message;
			} else {
				message = 'Unknown error';
			}

			pushError(
				{
					name: 'Failed to update Gale',
					message
				},
				true
			);
		}

		$nextUpdate = null;
	}

	async function update() {
		loading = true;
		await installUpdate();
		loading = false;

		if (platform() !== 'windows') {
			// on other platforms installUpdate() relaunches the app itself
			await message('Gale will now restart in order to apply the update.', {
				title: 'Update installed'
			});
			await relaunch();
		}
	}
</script>

{#if $nextUpdate != null}
	<Button.Root
		class="bg-accent-600 enabled:hover:bg-accent-500 my-auto ml-auto mr-1.5 flex items-center 
            rounded-md px-2.5 py-1 font-semibold text-gray-100"
		disabled={loading}
		on:click={() => (popupOpen = true)}
	>
		{#if loading}
			<Icon icon="mdi:loading" class="mr-1 animate-spin" />
		{:else}
			<Icon icon="mdi:arrow-up-circle" class="mr-1" />
		{/if}
		<span class="text-sm">{loading ? 'Downloading update...' : 'Update available'}</span>
	</Button.Root>
{/if}

<ConfirmPopup title="App update available" bind:open={popupOpen}>
	<Dialog.Description class="text-gray-300">
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

	<svelte:fragment slot="buttons">
		<BigButton color="accent" fontWeight="semibold" on:click={update}>Install</BigButton>
	</svelte:fragment>
</ConfirmPopup>
