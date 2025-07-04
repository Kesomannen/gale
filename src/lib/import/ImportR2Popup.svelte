<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import type { R2ImportData } from '$lib/types';
	import ImportR2Flow from './ImportR2Flow.svelte';

	type Props = {
		open: boolean;
	};

	let { open = $bindable() }: Props = $props();

	let loading = $state(false);
	let importFlow: ImportR2Flow;
	let importData: R2ImportData | null = $state(null);

	$effect(() => {
		if (open && importFlow) {
			importFlow.refresh(null);
		}
	});

	async function doImport() {
		if (await importFlow.doImport()) {
			open = false;
		}
	}
</script>

<Popup bind:open title="Import profiles from other manager" canClose={!loading}>
	<div class="text-primary-300 mb-2">
		<p>
			This will import profiles <b>for the current game</b> from r2modman or Thunderstore Mod Manager.
		</p>

		<p class="mt-2">
			<b>Do not close Gale while the import is in progress.</b>
		</p>
	</div>
	<ImportR2Flow bind:this={importFlow} bind:loading bind:importData />

	<div class="mt-3 mr-0.5 flex w-full justify-end gap-2">
		<Button color="primary" onclick={() => (open = false)}>Cancel</Button>
		<Button color="accent" onclick={doImport} icon="mdi:import">Import</Button>
	</div>
</Popup>
