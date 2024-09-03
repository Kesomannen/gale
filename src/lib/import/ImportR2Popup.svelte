<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import ImportR2Flow from './ImportR2Flow.svelte';
	import type { R2ImportData } from '$lib/models';

	import { t } from '$i18n';

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

<Popup bind:open title="{t('Import from R2')}" canClose={!loading} maxWidth="[55%]">
	<div class="text-slate-300">
		<p>{@html t('Import from R2 description 1')}</p>

		<p class="mt-2">{@html t('Import from R2 description 2')}</p>

		<p class="mt-2">{@html t('Import from R2 description 3')}</p>
	</div>

	<ImportR2Flow bind:this={importFlow} bind:importData bind:importFrom bind:loading />

	<div class="flex gap-2 justify-end w-full mr-0.5 mt-3">
		{#if importData?.r2modman || importData?.thunderstore}
			<BigButton color="gray" on:click={() => (open = false)}>{t("Cancel")}</BigButton>
			<BigButton color="green" on:click={doImport}>{t("Import")}</BigButton>
		{/if}
	</div>
</Popup>
