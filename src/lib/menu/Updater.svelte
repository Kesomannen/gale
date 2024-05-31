<script lang="ts">
	import { updated } from '$app/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { pushError } from '$lib/invoke';
	import Icon from '@iconify/svelte';
	import { dialog, os } from '@tauri-apps/api';
	import { getVersion } from '@tauri-apps/api/app';
	import type { UnlistenFn } from '@tauri-apps/api/event';
	import { platform } from '@tauri-apps/api/os';
	import { relaunch } from '@tauri-apps/api/process';
	import {
		checkUpdate,
		installUpdate,
		onUpdaterEvent,
		type UpdateManifest,
		type UpdateStatus
	} from '@tauri-apps/api/updater';
	import { Button, Dialog } from 'bits-ui';
	import { onMount } from 'svelte';

	let updateAvailable = false;
	let manifest: UpdateManifest | undefined = {
		body: '',
		date: '',
		version: '0.5.1'
	};

	let currentVersion = '1.0.0';

	let popupOpen = false;
	let loading = false;

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		onUpdaterEvent(({ error, status }) => {
			console.log(status);

			if (error) {
				pushError(
					{
						name: 'Failed to update Gale',
						message: error
					},
					true
				);
			}
		}).then((unlistenFn) => (unlisten = unlistenFn));

		checkUpdate().then((result) => {
			updateAvailable = result.shouldUpdate;
			manifest = result.manifest;
		});

		getVersion().then((version) => {
			currentVersion = version;
		});

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});

	async function update() {
    loading = true;
		await installUpdate();
    loading = false;

		let platformName = await platform();
		if (platformName !== 'win32') {
			await dialog.message('Gale will now restart in order to apply the update.', {
				title: 'Update installed'
			});
			await relaunch();
		}
	}
</script>

{#if updateAvailable}
	<Button.Root
		class="flex justify-center items-center h-9 w-9 rounded-lg font-semibold text-slate-100 
            my-auto ml-auto mr-1.5 bg-blue-600 hover:bg-blue-500"
    disabled={loading}
		on:click={() => (popupOpen = true)}
	>
		{#if loading}
			<Icon icon="mdi:loading" class="animate-spin" />
		{:else}
			<Icon icon="mdi:arrow-up-circle" class="text-xl" />
		{/if}
	</Button.Root>
{/if}

<ConfirmPopup title="App update available" bind:open={popupOpen}>
	<Dialog.Description class="text-slate-300">
		<p>
			{#if manifest}
				Version {manifest.version} of Gale is available - you have {currentVersion}.
			{:else}
				There is an update available for Gale.
			{/if}

			The update will be downloaded and installed in the background, and then the app will restart
			to apply the changes.
		</p>
		<p class="mt-1">Would you like to install it?</p>
	</Dialog.Description>

	<svelte:fragment slot="buttons">
		<BigButton color="blue" fontWeight="semibold" onClick={update}>Install</BigButton>
	</svelte:fragment>
</ConfirmPopup>
