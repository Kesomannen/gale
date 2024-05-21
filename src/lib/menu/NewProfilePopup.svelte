<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import { Button, Dialog } from 'bits-ui';
	import { refreshProfiles } from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';

	export let open = false;

	let newProfileName = '';

	async function createProfile() {
		if (newProfileName.length === 0) return;

		invokeCommand('create_profile', { name: newProfileName }).then(() => refreshProfiles());
		newProfileName = '';
	}
</script>

<Popup title="New Profile" bind:open>
	<Dialog.Description class="text-slate-400 mb-1">Enter a name for the new profile</Dialog.Description>
	<InputField bind:value={newProfileName} placeholder="Enter profile name..." size='lg' />
	<div class="flex w-full justify-end mt-1">
		<Dialog.Close>
			<BigButton disabled={newProfileName.length === 0} onClick={createProfile}>Create</BigButton>
		</Dialog.Close>
	</div>
</Popup>
