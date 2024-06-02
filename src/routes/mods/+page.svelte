<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import { SortBy, type Mod, type QueryModsArgs } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { fly } from 'svelte/transition';
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { modQuery } from '$lib/stores';
	import type { PageData } from './$types';

	const sortOptions = [SortBy.LastUpdated, SortBy.Newest, SortBy.Rating, SortBy.Downloads];

	export let data: PageData;

	let mods = data.mods;
	let activeMod: Mod | undefined;
	let activeDownloadSize: number | undefined;

	let versionsDropdownOpen = false;

	let missingDepsOpen = false;
	let missingDeps: string[] = [];

	$: activeModRef = activeMod
		? {
				packageUuid: activeMod.uuid,
				versionUuid: activeMod.versions[0].uuid
			}
		: undefined;

	let isActiveModInstalled = false;

	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			mods = evt.payload;
		}).then((unlisten) => {
			unlistenFromQuery = unlisten;
		});

		return () => {
			if (unlistenFromQuery) {
				unlistenFromQuery();
			}
			invokeCommand('stop_querying_thunderstore');
		};
	});

	$: if (activeMod) {
		invokeCommand<boolean>('is_mod_installed', { uuid: activeMod.uuid }).then(
			(result) => (isActiveModInstalled = result)
		);

		invokeCommand<number>('get_download_size', { modRef: activeModRef }).then(
			(size) => (activeDownloadSize = size)
		);
	}

	$: {
		$modQuery;
		refresh();
	}

	async function refresh() {
		mods = await invokeCommand<Mod[]>('query_thunderstore', { args: $modQuery });
	}

	async function install(modRef?: { packageUuid: string; versionUuid: string }) {
		missingDeps = await invokeCommand<string[]>('missing_deps', { modRef });
		if (missingDeps.length > 0) {
			missingDepsOpen = true;
			return;
		}

		await invokeCommand('install_mod', { modRef });
	}
</script>

<ModList bind:activeMod bind:mods queryArgs={modQuery} {sortOptions}>
	<div slot="details" class="flex mt-2 text-lg text-white">
		<Button.Root
			class="flex items-center justify-center flex-grow gap-2 py-2 rounded-l-lg
								enabled:bg-green-600 enabled:hover:bg-green-500 enabled:font-semibold
								disabled:bg-gray-600 disabled:opacity-80 disabled:cursor-not-allowed"
			on:click={() => install(activeModRef)}
			disabled={isActiveModInstalled}
		>
			{#if isActiveModInstalled}
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
				disabled={isActiveModInstalled}
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
				transition={fly}
				transitionConfig={{ duration: 100 }}
			>
				{#each activeMod?.versions ?? [] as version}
					<DropdownMenu.Item
						class="flex flex-shrink-0 items-center px-3 py-1 truncate text-slate-300 hover:text-slate-100 text-left rounded-md hover:bg-gray-600 cursor-default"
						on:click={() => {
							if (!activeMod) return;

							install({
								packageUuid: activeMod.uuid,
								versionUuid: version.uuid
							});
						}}
					>
						{version.name}
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>

	<div slot="item" let:mod>
		<Button.Root
			class="bg-green-600 hover:bg-green-500 p-2.5 ml-2 mt-0.5 mr-0.5 
							rounded-lg text-white text-2xl align-middle hidden group-hover:inline"
			on:click={() => {
				let modRef = {
					packageUuid: mod.uuid,
					versionUuid: mod.versions[0].uuid
				};

				install(modRef);
			}}
		>
			<Icon icon="mdi:download" />
		</Button.Root>
	</div>
</ModList>

<ConfirmPopup title="Missing dependencies" bind:open={missingDepsOpen}>
	Some of {activeMod?.name}'s dependencies could not be found:

	<ul class="mt-1">
		{#each missingDeps as dep}
			<li>- {dep}</li>
		{/each}
	</ul>

	<svelte:fragment slot="buttons">
		<BigButton
			color="red"
			fontWeight="semibold"
			onClick={() => {
				invokeCommand('install_mod', { modRef: activeModRef });
			}}
		>
			Install anyway
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>
