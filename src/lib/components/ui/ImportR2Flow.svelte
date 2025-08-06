<script lang="ts">
	import type { R2ImportData } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { fade } from 'svelte/transition';
	import Checklist from '$lib/components/ui/Checklist.svelte';
	import PathPref from '$lib/components/prefs/PathPref.svelte';
	import { capitalize } from '$lib/util';
	import * as api from '$lib/api';
	import { default as profileState } from '$lib/state/profile.svelte';
	import Spinner from './Spinner.svelte';

	let path: string | null = $state(null);
	let error = $state('');

	type Props = {
		importData?: R2ImportData | null | undefined;
		loading?: boolean;
	};

	let { importData = $bindable(undefined), loading = $bindable(false) }: Props = $props();

	let profiles = $derived(importData?.profiles ?? []);
	let include = $derived(importData?.include ?? []);

	$effect(() => {
		path = importData?.path ?? null;

		if (importData) {
			importData.include = importData.profiles.map(() => true);
		}
	});

	export async function refresh(newPath: string | null) {
		error = '';
		path = newPath;

		try {
			importData = await api.profile.import.getR2modmanInfo(newPath);
		} catch (e) {
			importData = null;
			error = e as string;

			console.error(error);
		}
	}

	export async function doImport() {
		if (!importData) return;

		loading = true;

		let success = false;

		try {
			await api.profile.import.r2modman(importData.path, include);

			success = true;
		} finally {
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
		class="text-primary-300 absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-black/60 text-xl"
		transition:fade={{ duration: 50 }}
	>
		<Spinner />
	</div>
{/if}

{#if importData === undefined}
	<div class="text-primary-300">Loading...</div>
{:else if importData === null}
	<div class="text-primary-300 mt-2 flex w-full items-center gap-2">
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
