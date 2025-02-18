<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { R2ImportData } from '$lib/models';
	import { refreshProfiles } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { fade } from 'svelte/transition';
	import Checklist from '$lib/components/Checklist.svelte';
	import PathPref from '$lib/prefs/PathPref.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { capitalize } from '$lib/util';

	export let importData: R2ImportData | null | undefined = undefined;

	let path: string | null = null;
	let error = '';

	export let loading = false;
	let loadingText = '';

	$: profiles = importData?.profiles ?? [];
	$: include = importData?.include ?? [];

	$: {
		path = importData?.path ?? null;

		if (importData) {
			importData.include = importData.profiles.map(() => true);
		}
	}

	export async function refresh(newPath: string | null) {
		console.log('refreshing path', newPath);

		error = '';
		path = newPath;

		try {
			importData = await invoke<R2ImportData | null>('get_r2modman_info', { path: newPath });
		} catch (e) {
			importData = null;
			error = e as string;

			console.error(error);
		}
	}

	export async function doImport() {
		if (importData === null) {
			return;
		}

		loading = true;

		let unlisten = await listen<string>('transfer_update', (evt) => {
			loadingText = evt.payload;
		});

		let success = false;

		try {
			await invokeCommand('import_r2modman', importData);
			refreshProfiles();

			success = true;
		} finally {
			unlisten();

			loading = false;
		}

		return success;
	}
</script>

<PathPref label="R2 data folder" type="dir" value={path} set={refresh}>
	The data folder of your r2modman/TMM installation.
</PathPref>

{#if loading}
	<div
		class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-black/60"
		transition:fade={{ duration: 50 }}
	>
		<Icon icon="mdi:loading" class="animate-spin text-4xl text-slate-300" />
		<div class="text-slate-300">{loadingText}</div>
	</div>
{/if}

{#if importData === undefined}
	<div class="text-slate-300">Loading...</div>
{:else if importData === null}
	<div class="mt-2 flex w-full items-center gap-2 text-slate-300">
		{#if path === null}
			<Icon icon="mdi:warning" />
			No installations found, please specify the path above.
		{:else}
			<Icon icon="mdi:error" class="shrink-0" />
			{capitalize(error)}
		{/if}
	</div>
{:else}
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
