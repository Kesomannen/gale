<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import { Button, Dialog } from 'bits-ui';
	import { refreshProfiles } from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';

	export let open = false;

	let newProfileName = '';

	async function createProfile() {
		if (newProfileName.length === 0) return;

		invokeCommand('create_profile', { name: newProfileName }).then(() => refreshProfiles());
		newProfileName = '';
	}
</script>

<Popup title="New Profile" bind:open>
	<Dialog.Description class="text-slate-400">Enter a name for the new profile</Dialog.Description>
	<input
		type="text"
		bind:value={newProfileName}
		on:keydown={(e) => {
			if (e.key !== 'Enter') return;
			createProfile();
			open = false;
		}}
		placeholder="Profile name"
		class="w-full my-2 px-3 py-2 rounded-lg bg-gray-900 text-slate-100 select-none"
	/>
	<div class="flex w-full justify-end mt-3">
		<Dialog.Close>
			<Button.Root
				class="rounded-xl px-4 py-2 mr-0.5 text-slate-100 bg-green-600 hover:bg-green-500 disabled:bg-gray-600/80 disabled:hover:bg-gray-600/80 disabled:text-gray-200/80"
				disabled={newProfileName.length === 0}
				on:click={createProfile}
			>
				Create
			</Button.Root>
		</Dialog.Close>
	</div>
</Popup>
