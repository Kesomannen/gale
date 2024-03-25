<script lang="ts">
	import InputField from '$lib/InputField.svelte';
	import PathField from '$lib/PathField.svelte';
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/error';
	import { currentProfile } from '$lib/profile';
	import { open } from '@tauri-apps/api/dialog';

	import { Button, Dialog } from 'bits-ui';

	export let isOpen: boolean = false;

	let name: string;
	let description: string;
	let version: string;
	let webiste: string;
	let iconPath: string;

	function browseIcon() {
		open({
			defaultPath: iconPath ?? undefined,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg'] }],
			title: 'Select icon',
			multiple: false
		}).then((result) => {
			if (result === null) return;
			iconPath = result as string;
		});
	}

	function submit() {
		open({
			title: 'Choose a location to save the modpack',
			directory: true
		}).then(async (result) => {
			if (result === null) return;
			let path = result as string;

			await invokeCommand('export_pack', {
				path,
				args: {
					name,
					description,
					version_number: version,
					website_url: webiste,
					icon: iconPath
				}
			})

			isOpen = false;
		});
	}
</script>

<Popup title="Export modpack" bind:open={isOpen}>
	<Dialog.Description class="text-slate-400">
		Export {$currentProfile} as a Thunderstore modpack
	</Dialog.Description>
	<div class="flex flex-col mt-4 gap-1">
		<InputField label="Name" placeholder="Enter a name..." pattern="[A-Z]" bind:value={name} />
		<InputField label="Description" placeholder="Enter a short description..." bind:value={description} />

		<div class="h-1" />

		<InputField label="Website" placeholder="Enter a website URL... (optional)" bind:value={webiste} />
		<InputField label="Version" placeholder="Version number (e.g. 1.2.3)" bind:value={version} />

		<div class="h-1" />

		<PathField label="Icon" onClick={browseIcon} bind:value={iconPath} icon="mdi:file-image" />
	</div>
	<div class="flex w-full justify-end mt-3">
		<Button.Root
			class="rounded-lg px-6 py-2 mr-0.5 text-white font-medium bg-green-700 hover:bg-green-600 disabled:bg-gray-600/80 disabled:hover:bg-gray-600/80 disabled:text-gray-200/80"
			on:click={submit}
		>
			Export
		</Button.Root>
	</div>
</Popup>
