<script lang="ts">
	import { refreshProfiles } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import Label from '$lib/components/Label.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';

	export let open = false;

	let name: string;
	let override = false;
	let path: string | null;

	$: if (open) name = '';

	async function createProfile() {
		await invokeCommand('create_profile', { name, overridePath: override ? path : null });
		refreshProfiles();
		open = false;
	}

	async function browse() {
		path = await openDialog({
			directory: true
		});
	}
</script>

<ConfirmPopup title="Create new profile" bind:open>
	Choose a name for the new profile

	<InputField
		placeholder="Enter name..."
		class="mt-1 w-full"
		on:submit={createProfile}
		bind:value={name}
	/>

	<div class="mt-2 mb-1 flex items-center">
		<Label>Use custom path</Label>
		<Checkbox bind:value={override} />
	</div>

	{#if override}
		<PathField label="Custom path" bind:value={path} on:click={browse}
			>The path of the profile.</PathField
		>
	{/if}

	<svelte:fragment slot="buttons">
		<BigButton on:click={createProfile}>Create</BigButton>
	</svelte:fragment>
</ConfirmPopup>
