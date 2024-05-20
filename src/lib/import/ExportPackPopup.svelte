<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
import InputField from '$lib/components/InputField.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { currentProfile } from '$lib/profile';
	import { open } from '@tauri-apps/api/dialog';

	import { Button, Dialog } from 'bits-ui';

	export let isOpen: boolean = false;

	let name: string;
	let description: string;
	let versionNumber: string;
	let websiteUrl: string;
	let icon: string;

	function browseIcon() {
		open({
			defaultPath: icon ?? undefined,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg'] }],
			title: 'Select icon',
			multiple: false
		}).then((result) => {
			if (result === null) return;
			icon = result as string;
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
					versionNumber,
					websiteUrl,
					icon
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
		<InputField label="Name" placeholder="Enter a name..." bind:value={name} />
		<InputField label="Description" placeholder="Enter a short description..." bind:value={description} />

		<div class="h-1" />

		<InputField label="Website" placeholder="Enter a website URL... (optional)" bind:value={websiteUrl} />
		<InputField label="Version" placeholder="Version number (e.g. 1.2.3)" bind:value={versionNumber} />

		<div class="h-1" />

		<PathField label="Icon" onClick={browseIcon} bind:value={icon} icon="mdi:file-image" />
	</div>
	<div class="flex w-full justify-end mt-3">
		<BigButton onClick={submit}>
			Export
		</BigButton>
	</div>
</Popup>
