<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import { Button, Dialog } from 'bits-ui';
	import { refreshProfiles } from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';

	export let open = false;

	let name = '';

	async function createProfile() {
		if (name.length === 0) return;

		await invokeCommand('create_profile', { name });
		refreshProfiles();
		name = '';
		open = false;
	}
</script>

<Popup title="New profile" bind:open>
	<div class="h-1" />
	<InputField
		bind:value={name}
		placeholder="Enter profile name..."
		size="lg"
		onSubmit={createProfile}
	/>
	<div class="flex w-full justify-end mt-1">
		<BigButton disabled={name.length === 0} onClick={createProfile}>Create</BigButton>
	</div>
</Popup>
