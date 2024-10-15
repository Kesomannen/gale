<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import { SortBy, type Mod } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { fly } from 'svelte/transition';
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { modQuery, activeGame } from '$lib/stores';
	import ModListItem from '$lib/modlist/ModListItem.svelte';

	const sortOptions = [SortBy.LastUpdated, SortBy.Newest, SortBy.Rating, SortBy.Downloads];

	let mods: Mod[] = [];
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
		invokeCommand<number>('get_download_size', { modRef: activeModRef }).then(
			(size) => (activeDownloadSize = size)
		);
	}

	$: {
		$modQuery;
		$activeGame;
		refresh();
	}

	async function refresh() {
		mods = await invokeCommand<Mod[]>('query_thunderstore', { args: $modQuery });
	}

	async function installLatest(mod: Mod) {
		await install({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
	}

	async function install(modRef?: { packageUuid: string; versionUuid: string }) {
		missingDeps = await invokeCommand<string[]>('get_missing_deps', { modRef });
		if (missingDeps.length > 0) {
			missingDepsOpen = true;
			return;
		}

		await invokeCommand('install_mod', { modRef });
		modQuery.update((query) => query);
		activeMod = activeMod;
	}
</script>

<ModList bind:activeMod bind:mods queryArgs={modQuery} {sortOptions}>
	<div slot="details" class="mt-2 flex text-lg text-white">
		<Button.Root
			class="flex flex-grow items-center justify-center gap-2 rounded-l-lg py-2 enabled:bg-green-600 enabled:font-semibold enabled:hover:bg-green-500 disabled:cursor-not-allowed disabled:bg-gray-600 disabled:opacity-80"
			on:click={() => install(activeModRef)}
			disabled={activeMod?.isInstalled}
		>
			{#if activeMod?.isInstalled}
				Mod already installed
			{:else}
				<Icon icon="mdi:download" class="align-middle text-xl" />
				Install
				{#if activeDownloadSize !== undefined && activeDownloadSize > 0}
					({shortenFileSize(activeDownloadSize)})
				{/if}
			{/if}
		</Button.Root>
		<DropdownMenu.Root bind:open={versionsDropdownOpen}>
			<DropdownMenu.Trigger
				class="ml-0.5 gap-2 rounded-r-lg px-1.5 py-2 text-2xl enabled:bg-green-600 enabled:font-medium enabled:hover:bg-green-500 disabled:cursor-not-allowed disabled:bg-gray-600 disabled:opacity-80"
				disabled={activeMod?.isInstalled}
			>
				<Icon
					icon="mdi:chevron-down"
					class="origin-center transform align-middle text-xl transition-transform {versionsDropdownOpen
						? 'rotate-180'
						: 'rotate-0'}"
				/>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content
				class="flex max-h-72 w-48 flex-col gap-0.5 overflow-y-auto rounded-lg border border-gray-500 bg-gray-700 p-1 shadow-xl"
				transition={fly}
				transitionConfig={{ duration: 100 }}
			>
				{#each activeMod?.versions ?? [] as version}
					<DropdownMenu.Item
						class="flex flex-shrink-0 cursor-default items-center truncate rounded-md px-3 py-1 text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
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

	<div slot="item" let:mod let:isSelected>
		<ModListItem {mod} {isSelected} on:install={({ detail: { mod } }) => installLatest(mod)} />
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
			on:click={() => {
				invokeCommand('install_mod', { modRef: activeModRef });
			}}
		>
			Install anyway
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>
