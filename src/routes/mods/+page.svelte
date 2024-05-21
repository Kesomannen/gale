<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { Mod, QueryModsArgs } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { currentGame } from '$lib/profile';
	import { slide } from 'svelte/transition';

	let mods: Mod[];
	let queryArgs: QueryModsArgs;
	let activeMod: Mod | undefined;
	let activeDownloadSize: number | undefined;

	let versionsDropdownOpen = false;

	$: modRef = {
		packageUuid: activeMod?.uuid,
		versionUuid: activeMod?.versions[0].uuid
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

	$: if (activeMod) {
		invokeCommand<boolean>('is_mod_installed', { uuid: activeMod.uuid }).then(
			(result) => (isModInstalled = result)
		);

		invokeCommand<number>('get_download_size', { modRef }).then(
			(size) => (activeDownloadSize = size)
		);
	}
</script>

<ModList bind:activeMod bind:mods bind:queryArgs>
	<div slot="details" class="flex mt-2 text-lg text-white">
		<Button.Root
			class="flex items-center justify-center flex-grow gap-2 py-2 rounded-l-lg
								enabled:bg-green-600 enabled:hover:bg-green-500 enabled:font-semibold
								disabled:bg-gray-600 disabled:opacity-80 disabled:cursor-not-allowed"
			on:click={() => {
				if (activeMod) {
					invokeCommand('install_mod', { modRef });
				}
			}}
			disabled={isModInstalled}
		>
			{#if isModInstalled}
				Mod already installed
			{:else}
				<Icon icon="mdi:download" class="text-xl align-middle" />
				Install
				{#if activeDownloadSize !== undefined && activeDownloadSize > 0}
					({shortenFileSize(activeDownloadSize)})
				{/if}
			{/if}
		</Button.Root>
		<DropdownMenu.Root bind:open={versionsDropdownOpen}>
			<DropdownMenu.Trigger
				class="gap-2 rounded-r-lg py-2 px-1.5 ml-0.5 text-2xl
						enabled:bg-green-600 enabled:hover:bg-green-500 enabled:font-medium
						disabled:bg-gray-600 disabled:opacity-80 disabled:cursor-not-allowed"
				disabled={isModInstalled}
			>
				<Icon
					icon="mdi:chevron-down"
					class="text-xl align-middle transform transition-transform origin-center {versionsDropdownOpen
						? 'rotate-180'
						: 'rotate-0'}"
				/>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content
				class="flex flex-col bg-gray-700 gap-0.5 shadow-xl p-1 w-48 rounded-lg border border-gray-500 max-h-72 overflow-y-auto"
			>
				{#each activeMod?.versions ?? [] as version}
					<DropdownMenu.Item
						class="flex flex-shrink-0 items-center px-3 py-1 truncate text-slate-300 hover:text-slate-100 text-left rounded-md hover:bg-gray-600 cursor-default"
						on:click={() => {
							let versionedModRef = {
								packageUuid: activeMod?.uuid,
								versionUuid: version.uuid
							};

							invokeCommand('install_mod', { modRef: versionedModRef });
						}}
					>
						{version.name}
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>
</ModList>
