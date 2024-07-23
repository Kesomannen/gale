<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import { refreshProfiles } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';

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

<ConfirmPopup title="Create new profile" bind:open>
	<p class="mb-1">Enter a name for the new profile:</p>
	<InputField
		placeholder="Enter profile name..."
		size="lg"
		class="w-full"
		on:submit={createProfile}
		bind:value={name}
	/>
	<svelte:fragment slot="buttons">
		<BigButton disabled={name.length === 0} on:click={createProfile}>Create</BigButton>
	</svelte:fragment>
</ConfirmPopup>
