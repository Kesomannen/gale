<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { refreshProfiles } from '$lib/profile';
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { slide } from 'svelte/transition';

	export let open: boolean;

	let importing = false;
	let message: string | undefined;

	async function doImport() {
		importing = true;

		let unlisten = await listen<string>('transfer_update', (evt) => {
			message = evt.payload;
		});

		open = false;

		try {
			await invokeCommand('import_r2modman');
			refreshProfiles();
		} finally {
			unlisten();

			importing = false;
			message = undefined;
		}
	}
</script>

<Popup bind:open title="Import profiles from r2modman" canClose={!importing} maxWidth="[55%]">
	<div class="text-slate-300">
		<p>
			This will import cached mods and profiles <b>for the current game</b> from r2modman or
			Thunderstore Mod Manager.
		</p>

		<p class="mt-2">
			The process may take a couple of minutes, depending on how many mods
			and profiles there are to import.
			<b>Profiles with the same name will be overwritten!</b>
		</p>

		<p class="mt-2">Please do not close Gale while the import is in progress.</p>
	</div>

	{#if !importing}
		<div class="flex gap-2 justify-end w-full mr-0.5 mt-3">
			<BigButton color="gray" onClick={() => (open = false)}>Cancel</BigButton>
			<BigButton color="green" onClick={doImport}>Import</BigButton>
		</div>
	{/if}
</Popup>

{#if message}
	<div
		class="flex items-center bottom-2 right-2 left-2 rounded-lg p-2 absolute z-10 bg-gray-800
              border border-gray-700 text-slate-400"
		transition:slide
	>
		<Icon icon="mdi:loading" class="animate-spin text-2xl mr-2" />
		{message}
	</div>
{/if}
