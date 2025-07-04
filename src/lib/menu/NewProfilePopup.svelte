<script lang="ts">
	import { refreshProfiles } from '$lib/stores.svelte';
	import * as api from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import Label from '$lib/components/Label.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';

	type Props = {
		open?: boolean;
	};

	let { open = $bindable(false) }: Props = $props();

	let name: string = $state('');
	let override = $state(false);
	let path: string | null = $state(null);

	$effect(() => {
		if (open) name = '';
	});

	async function createProfile() {
		await api.profile.create(name, override ? path : null);
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
		onsubmit={createProfile}
		bind:value={name}
	/>

	<div class="mt-2 mb-1 flex items-center">
		<Label>Use custom path</Label>
		<Checkbox bind:checked={override} />
	</div>

	{#if override}
		<PathField label="Custom path" bind:value={path} onclick={browse}
			>The path of the profile.</PathField
		>
	{/if}

	{#snippet buttons()}
		<Button onclick={createProfile} icon="mdi:plus">Create</Button>
	{/snippet}
</ConfirmPopup>
