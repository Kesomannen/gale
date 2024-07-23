<script context="module" lang="ts">
	import { check, type Update } from '@tauri-apps/plugin-updater';

	export let updateAvailable = false;
	export let currentUpdate: Update | undefined;

	export async function refreshUpdate() {
		let update = await check();
		if (update?.available) {
			updateAvailable = true;
			currentUpdate = update;
		}
	}
</script>

<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { pushError } from '$lib/invoke';
	import Icon from '@iconify/svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { getVersion } from '@tauri-apps/api/app';
	import { platform } from '@tauri-apps/plugin-os';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Button, Dialog } from 'bits-ui';
	import { onMount } from 'svelte';

	let currentVersion = '1.0.0';

	let popupOpen = false;
	let loading = false;

	onMount(() => {
		refreshUpdate();

		getVersion().then((version) => {
			currentVersion = version;
		});
	});

	async function installUpdate() {
		if (!currentUpdate) return;

		try {
			await currentUpdate.downloadAndInstall();
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

		currentUpdate = undefined;
	}

	async function update() {
		loading = true;
		await installUpdate();
		loading = false;

		let platformName = await platform();
		if (platformName !== 'windows') { // on other platforms installUpdate() relaunches the app itself
			await message('Gale will now restart in order to apply the update.', {
				title: 'Update installed'
			});
			await relaunch();
		}
	}
</script>

{#if updateAvailable}
	<Button.Root
		class="flex items-center py-1 px-2 rounded-md font-semibold text-slate-100 
            my-auto ml-auto mr-1.5 bg-blue-600 enabled:hover:bg-blue-500"
    	disabled={loading}
		on:click={() => (popupOpen = true)}
	>
		{#if loading}
			<Icon icon="mdi:loading" class="animate-spin mr-1" />
		{:else}
			<Icon icon="mdi:arrow-up-circle" class="mr-1" />
		{/if}
		<span class="text-sm">{loading ? "Downloading update..." : "Update available"}</span>
	</Button.Root>
{/if}

<ConfirmPopup title="App update available" bind:open={popupOpen}>
	<Dialog.Description class="text-slate-300">
		<p>
			{#if currentUpdate}
				Version {currentUpdate.version} of Gale is available - you have {currentVersion}.
			{:else}
				There is an update available for Gale.
			{/if}

			The update will be downloaded and installed in the background, and then the app will restart
			to apply the changes.
		</p>
		<p class="mt-1">Would you like to install it?</p>
	</Dialog.Description>

	<svelte:fragment slot="buttons">
		<BigButton color="blue" fontWeight="semibold" on:click={update}>Install</BigButton>
	</svelte:fragment>
</ConfirmPopup>
