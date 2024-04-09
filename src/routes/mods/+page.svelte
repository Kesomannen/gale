<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { Mod, QueryModsArgs } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { currentGame } from '$lib/profile';

	let mods: Mod[];
	let queryArgs: QueryModsArgs;
	let activeMod: Mod | undefined;
	let activeDownloadSize: number | undefined;

	$: modRef = {
		packageUuid: activeMod?.uuid,
		versionUuid: activeMod?.latestVersionUuid,
	};

	let isModInstalled = false;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			mods = evt.payload;
		});
	});

	$: {
		$currentGame;
		if (queryArgs) {
			invokeCommand<Mod[] | null>('query_all_mods', { args: queryArgs }).then((result) => {
				if (!result) return;
				mods = result;
			});
		}
	}

	$: {
		if (activeMod) {
			invokeCommand<number>('get_download_size', { modRef }).then(
				(size) => (activeDownloadSize = size)
			);
		}
	}

	$: if (activeMod) {
		invokeCommand<boolean>('is_mod_installed', { uuid: activeMod.uuid }).then(
			(result) => (isModInstalled = result)
		);
	}
</script>

<ModList bind:activeMod bind:mods bind:queryArgs>
	<Button.Root
		slot="details"
		class="flex items-center justify-center gap-2 py-2 mt-2 rounded-lg text-lg text-slate-100
								enabled:bg-green-600 enabled:hover:bg-green-500 enabled:font-medium
								disabled:bg-gray-600 disabled:opacity-80 disabled:cursor-not-allowed"
		on:click={() => {
			if (activeMod) {
				invokeCommand('install_mod', { modRef })
			}
		}}
		disabled={isModInstalled}
	>
		{#if isModInstalled}
			Mod already installed
		{:else}
			<Icon icon="mdi:download" class="text-white text-xl align-middle" />
			Install
			{#if activeDownloadSize !== undefined && activeDownloadSize > 0}
				({shortenFileSize(activeDownloadSize)})
			{/if}
		{/if}
	</Button.Root>
</ModList>
