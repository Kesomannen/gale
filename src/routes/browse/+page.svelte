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
	import { modQuery, activeProfileLocked, activeProfile } from '$lib/stores';
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ProfileLockedBanner from '$lib/modlist/ProfileLockedBanner.svelte';

	const sortOptions = [SortBy.LastUpdated, SortBy.Newest, SortBy.Rating, SortBy.Downloads];

	let mods: Mod[] = [];

	let modList: ModList;
	let maxCount: number;
	let selectedMod: Mod | null = null;
	let selectedDownloadSize: number | null = null;

	let versionsDropdownOpen = false;

	$: activeModRef = selectedMod
		? {
				packageUuid: selectedMod.uuid,
				versionUuid: selectedMod.versions[0].uuid
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
			if (unlistenFromQuery !== undefined) {
				unlistenFromQuery();
			}
			invokeCommand('stop_querying_thunderstore');
		};
	});

	$: if (selectedMod) {
		invokeCommand<number>('get_download_size', { modRef: activeModRef }).then(
			(size) => (selectedDownloadSize = size)
		);
	}

	$: if (maxCount > 0) {
		$modQuery;
		$activeProfile;
		refresh();
	}

	let hasRefreshed = false;
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await invokeCommand<Mod[]>('query_thunderstore', { args: { ...$modQuery, maxCount } });
		if (selectedMod !== null) {
			// isInstalled might have changed
			selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
		}

		refreshing = false;
		hasRefreshed = true;
	}

	async function installLatest(mod: Mod) {
		await install({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
	}

	async function install(modRef?: { packageUuid: string; versionUuid: string }) {
		await invokeCommand('install_mod', { modRef });
		await refresh();
	}

	function onModClicked(evt: MouseEvent, mod: Mod) {
		if (evt.ctrlKey) {
			installLatest(mod);
		} else {
			modList.selectMod(mod);
		}
	}
</script>

<ModList
	{sortOptions}
	queryArgs={modQuery}
	locked={$activeProfileLocked}
	bind:this={modList}
	bind:mods
	bind:maxCount
	bind:selected={selectedMod}
>
	<div slot="details" class="mt-2 flex text-lg text-white">
		<Button.Root
			class="enabled:bg-accent-600 enabled:hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 flex grow items-center justify-center gap-2 rounded-l-lg py-2 font-semibold disabled:cursor-not-allowed"
			on:click={() => install(activeModRef)}
			disabled={selectedMod?.isInstalled || $activeProfileLocked}
		>
			{#if $activeProfileLocked}
				Profile locked
			{:else if selectedMod?.isInstalled}
				Already installed
			{:else}
				<Icon icon="mdi:download" class="align-middle text-xl" />
				Install
				{#if selectedDownloadSize !== null && selectedDownloadSize > 0}
					({shortenFileSize(selectedDownloadSize)})
				{/if}
			{/if}
		</Button.Root>
		<DropdownMenu.Root bind:open={versionsDropdownOpen}>
			<DropdownMenu.Trigger
				class="enabled:bg-accent-600 enabled:hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 ml-0.5 gap-2 rounded-r-lg px-1.5 py-2 text-2xl disabled:cursor-not-allowed"
				disabled={selectedMod?.isInstalled || $activeProfileLocked}
			>
				<Icon
					icon="mdi:chevron-down"
					class="origin-center transform align-middle text-xl transition-transform {versionsDropdownOpen
						? 'rotate-180'
						: 'rotate-0'}"
				/>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content
				class="border-primary-500 bg-primary-700 flex max-h-72 w-48 flex-col gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
				transition={fly}
				transitionConfig={{ duration: 100 }}
			>
				{#each selectedMod?.versions ?? [] as version}
					<DropdownMenu.Item
						class="text-primary-300 hover:bg-primary-600 hover:text-primary-100 flex shrink-0 cursor-default items-center truncate rounded-md px-3 py-1 text-left"
						on:click={() => {
							if (!selectedMod) return;

							install({
								packageUuid: selectedMod.uuid,
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

	<svelte:fragment slot="banner">
		{#if $activeProfileLocked}
			<ProfileLockedBanner class="mr-4 mb-1" />
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="placeholder">
		{#if hasRefreshed}
			<span class="text-lg">No matching mods found</span>
			<br />
			<span class="text-primary-400">Try to adjust your search query/filters</span>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="item" let:data={{ mod, isSelected }}>
		<ModListItem
			{mod}
			{isSelected}
			locked={$activeProfileLocked}
			on:install={() => installLatest(mod)}
			on:click={(evt) => onModClicked(evt, mod)}
		/>
	</svelte:fragment>
</ModList>
