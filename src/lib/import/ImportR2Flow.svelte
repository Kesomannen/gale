<script lang="ts">
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';

	import { invokeCommand } from '$lib/invoke';
	import type { R2ImportData } from '$lib/models';
	import { refreshProfiles } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { fade } from 'svelte/transition';
	import Checklist from '$lib/components/Checklist.svelte';

	export let importData: R2ImportData = {
		r2modman: null,
		thunderstore: null
	};

	export let importFrom: 'r2modman' | 'thunderstore' = 'r2modman';

	$: profiles = importData[importFrom]?.profiles ?? [];
	$: include = importData[importFrom]?.include ?? [];

	export let loading = false;
	let loadingText = '';

	export async function doImport() {
		let data = importData[importFrom];

		if (!data) {
			return;
		}

		loading = true;

		let unlisten = await listen<string>('transfer_update', (evt) => {
			loadingText = evt.payload;
		});

		try {
			await invokeCommand('import_r2modman', { path: data.path, include: data.include });
			refreshProfiles();
		} finally {
			unlisten();

			loading = false;
		}
	}
</script>

{#if loading}
	<div
		class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-black/60"
		transition:fade={{ duration: 50 }}
	>
		<Icon icon="mdi:loading" class="animate-spin text-4xl text-slate-300" />
		<div class="text-slate-300">{loadingText}</div>
	</div>
{/if}

{#if !importData.r2modman && !importData.thunderstore}
	<div
		class="mt-3 flex w-full items-center justify-center gap-2 text-lg font-semibold text-red-400"
	>
		<Icon icon="mdi:error" />
		No installations found
	</div>
{/if}

{#if importData.r2modman && importData.thunderstore}
	<TabsMenu
		bind:value={importFrom}
		options={[
			{ value: 'r2modman', label: 'r2modman' },
			{ value: 'thunderstore', label: 'Thunderstore Mod Manager' }
		]}
	/>
{/if}

{#if importData.thunderstore || importData.r2modman}
	<Checklist
		class="mt-1"
		maxHeight="sm"
		items={profiles}
		title="Include all"
		getLabel={(item, _) => item}
		get={(_, index) => include[index]}
		set={(_, index, value) => (include[index] = value)}
	/>
{/if}
