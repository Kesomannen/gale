<script lang="ts">
	import { refreshProfiles } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';

	export let open = false;

	let name: string;

	$: if (open) name = '';

	async function createProfile() {
		if (name.length === 0) return;

		await invokeCommand('create_profile', { name });
		refreshProfiles();
		open = false;
	}
</script>

<ConfirmPopup title="Create new profile" bind:open>
	Choose a name for the new profile:
	<InputField
		placeholder="Enter name..."
		class="mt-1 w-full"
		on:submit={createProfile}
		bind:value={name}
	/>
	<svelte:fragment slot="buttons">
		<BigButton on:click={createProfile}>Create</BigButton>
	</svelte:fragment>
</ConfirmPopup>
