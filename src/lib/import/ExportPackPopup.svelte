<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { currentProfile } from '$lib/stores';
	import { open } from '@tauri-apps/api/dialog';

	import { Button, Dialog } from 'bits-ui';
	import { get } from 'svelte/store';

	export let isOpen: boolean = false;

	let name: string;
	let description: string;
	let versionNumber: string;
	let websiteUrl: string;
	let icon: string | null;

	$: if (isOpen) {
		reset();
	}

	$: isValid =
		name?.length > 0 &&
		!name.includes(' ') &&
		description?.length > 0 &&
		versionNumber?.length > 0 &&
		icon;

	function reset() {
		name = get(currentProfile).name;
		description = '';
		versionNumber = '1.0.0';
		websiteUrl = '';
		icon = null;
	}

	function browseIcon() {
		open({
			defaultPath: icon ?? undefined,
			filters: [{ name: 'Images', extensions: ['png'] }],
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
			});

			isOpen = false;
		});
	}
</script>

<Popup title="Export modpack" bind:open={isOpen}>
	<Dialog.Description class="text-slate-400">
		Export {$currentProfile.name} as a Thunderstore modpack
	</Dialog.Description>
	<div class="flex flex-col mt-4 gap-1">
		<InputField label="Name" placeholder="Enter a name..." bind:value={name}>
			The name that will be displayed on Thunderstore. Cannot contain spaces.
		</InputField>

		<InputField
			label="Description"
			placeholder="Enter a short description..."
			bind:value={description}
		>
			A short description of the modpack. This will also be the contents of the README file.
		</InputField>

		<div class="h-1" />

		<InputField
			label="Website"
			placeholder="Enter a website URL... (optional)"
			bind:value={websiteUrl}
		>
			A hyperlink to a website of your choosing. Optional.
		</InputField>

		<InputField label="Version" placeholder="Enter a version number..." bind:value={versionNumber}>
			The version number of the modpack. Must be in the format <b>x.y.z</b> and preferably follow
			<a class="text-green-500 hover:text-green-400 hover:underline" href="https://semver.org/"
				>semantic versioning</a
			>.
		</InputField>

		<div class="h-1" />

		<PathField label="Icon" onClick={browseIcon} bind:value={icon} icon="mdi:file-image">
			The icon that will be displayed on Thunderstore. Will be resized to 256x256, so it's
			recommended you enter a square image to avoid squashing/stretching. Must be in PNG format.
		</PathField>
	</div>
	<div class="flex w-full justify-end mt-3">
		<BigButton onClick={submit} disabled={!isValid}>Export</BigButton>
	</div>
</Popup>
