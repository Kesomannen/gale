<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { R2ImportData } from '$lib/models';
	import ImportR2Flow from './ImportR2Flow.svelte';

	export let open: boolean;

	let loading = false;
	let importFlow: ImportR2Flow;
	let importData: R2ImportData;

	$: if (open) {
		onOpen();
	}

	async function onOpen() {
		importData = await invokeCommand('get_r2modman_info');
		console.log(importData);
	}

	async function doImport() {
		await importFlow.doImport();
		open = false;
	}
</script>

<Popup bind:open title="Import profiles from other manager" canClose={!loading}>
	<div class="mb-2 text-slate-300">
		<p>
			This will import profiles <b>for the current game</b> from r2modman or Thunderstore Mod Manager.
		</p>

		<p class="mt-2">
			The process may take a couple of minutes, depending on how many mods there are to import.
			<b>Existing profiles with the same name will be overwritten!</b>
		</p>

		<p class="mt-2">Do not close Gale while the import is in progress.</p>
	</div>
	<ImportR2Flow bind:this={importFlow} bind:loading />

	<div class="mr-0.5 mt-3 flex w-full justify-end gap-2">
		<BigButton color="slate" on:click={() => (open = false)}>Cancel</BigButton>
		<BigButton color="accent" on:click={doImport}>Import</BigButton>
	</div>
</Popup>
