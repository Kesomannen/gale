<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { R2ImportData } from '$lib/types';
	import ImportR2Flow from '../ui/ImportR2Flow.svelte';

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

<Dialog bind:open title={m.importR2Dialog_title()} canClose={!loading}>
	<div class="text-primary-300 mb-2">
		<p>
			{m.importR2Dialog_content_1()}<b>{m.importR2Dialog_content_2()}</b>{m.importR2Dialog_content_3()}
		</p>

		<p class="mt-2">
			<b>{m.importR2Dialog_content_4()}</b>
		</p>
	</div>
	<ImportR2Flow bind:this={importFlow} bind:loading bind:importData />

	<div class="mt-3 mr-0.5 flex w-full justify-end gap-2">
		<Button color="primary" onclick={() => (open = false)}>{m.importR2Dialog_button_cancel()}</Button>
		<Button color="accent" onclick={doImport} icon="mdi:import">{m.importR2Dialog_button_import()}</Button>
	</div>
</Dialog>
