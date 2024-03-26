<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { Mod, ModQueryArgs } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';
	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	let mods: Mod[];
	let queryArgs: ModQueryArgs;
	let activeMod: Mod | undefined;
	let activeDownloadSize: number | undefined;
	
	let installingMod: Mod | undefined;
	let isInstalling = false;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			mods = evt.payload;
			console.log('Received mods', mods);
		});
	})

	$: {
		if (queryArgs) {
			invokeCommand<Mod[] | null>('query_all_mods', { args: queryArgs })
				.then((result) => {
					if (!result) return;
					mods = result;
				})
		}
	}

	$: {
		if (activeMod) {
			invokeCommand<number>('get_download_size', { packageUuid: activeMod.package.uuid4 })
				.then((size) => activeDownloadSize = size)
		}
	}

	async function install(mod: Mod | undefined) {
		if (!mod) return;

		installingMod = mod;
		isInstalling = true;
		try {
			await invokeCommand('install_mod', { packageUuid: mod.package.uuid4 });
			await new Promise((r) => setTimeout(r, 1000));
		} finally {
			isInstalling = false;
			installingMod = undefined;
		}
	}
</script>

<ModList bind:activeMod={activeMod} bind:mods={mods} bind:queryArgs={queryArgs}>
	<Button.Root
		slot="details"
		class="flex items-center justify-center gap-2 rounded-lg text-lg font-medium text-slate-100
								bg-green-600 hover:bg-green-500 py-2"
		on:click={() => install(activeMod)}
	>
		<Icon icon="mdi:download" class="text-white text-xl align-middle" />
		Install
		{#if activeDownloadSize !== undefined && activeDownloadSize > 0}
			({shortenFileSize(activeDownloadSize)})
		{/if}
	</Button.Root>
</ModList>

<InstallProgressPopup modName={installingMod?.version.name ?? 'Unknown'} bind:open={isInstalling} />
