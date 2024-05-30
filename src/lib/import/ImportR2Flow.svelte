<script lang="ts">
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';

	import { invokeCommand } from '$lib/invoke';
	import type { R2ImportData } from '$lib/models';
	import { refreshProfiles } from '$lib/profile';
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { fade } from 'svelte/transition';

	export let importData: R2ImportData = {
		r2modman: undefined,
		thunderstore: undefined
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
		class="inset-0 absolute z-50 flex flex-col gap-3 items-center justify-center bg-black/60"
		transition:fade={{ duration: 50 }}
	>
		<Icon icon="mdi:loading" class="text-6xl text-slate-400 animate-spin" />
		<div class="text-lg font-bold text-slate-400">{loadingText}</div>
	</div>
{/if}

<h3 class="text-lg text-slate-200 font-semibold mt-3">Choose profiles to import</h3>

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
  <div class="flex flex-col max-h-60 overflow-y-auto text-slate-300">
    {#each profiles as profile, index}
      <div class="flex items-center justify-between py-1 px-1">
        {profile}

        <Checkbox
          value={include[index]}
          onValueChanged={(value) => (include[index] = value)}
        />
      </div>
    {/each}
  </div>
{/if}
