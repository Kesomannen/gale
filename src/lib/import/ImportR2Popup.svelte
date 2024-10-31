<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import ImportR2Flow from './ImportR2Flow.svelte';
	import type { R2ImportData } from '$lib/models';
	import { onMount } from 'svelte';

	export let open: boolean;

	let loading = false;
	let importFlow: ImportR2Flow;
	let importData: R2ImportData;
	let importFrom: 'r2modman' | 'thunderstore';

	$: if (open) {
		load();
	}

	async function load() {
		importData = await invokeCommand<R2ImportData>('get_r2modman_info');

		if (importData.r2modman) {
			importData.r2modman.include = importData.r2modman.profiles.map(() => true);
			importFrom = 'r2modman';
		}

		if (importData.thunderstore) {
			importData.thunderstore.include = importData.thunderstore.profiles.map(() => true);
			importFrom = 'thunderstore';
		}
	}

	async function doImport() {
		await importFlow.doImport();
		open = false;
	}
</script>

<Popup bind:open title="Import profiles from other manager" canClose={!loading}>
	<div class="text-slate-300">
		<p>
			This will import profiles <b>for the current game</b> from r2modman or Thunderstore Mod Manager.
		</p>

		<p class="mt-2">
			The process may take a couple of minutes, depending on how many mods and profiles there are to
			import.
			<b>Profiles with the same name will be overwritten!</b>
		</p>

		<p class="mt-2">Please do not close Gale while the import is in progress.</p>
	</div>

	<ImportR2Flow bind:this={importFlow} bind:importData bind:importFrom bind:loading />

	<div class="mr-0.5 mt-3 flex w-full justify-end gap-2">
		{#if importData?.r2modman || importData?.thunderstore}
			<BigButton color="slate" on:click={() => (open = false)}>Cancel</BigButton>
			<BigButton color="accent" on:click={doImport}>Import</BigButton>
		{/if}
	</div>
</Popup>
