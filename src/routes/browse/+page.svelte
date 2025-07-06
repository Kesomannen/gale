<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId } from '$lib/types';

	import ModList from '$lib/components/mod-list/ModList.svelte';

	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { modQuery, activeProfileLocked, activeProfile } from '$lib/stores.svelte';
	import ModListItem from '$lib/components/mod-list/ModListItem.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import { defaultContextItems } from '$lib/context';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];
	const contextItems = [...defaultContextItems];

	let mods: Mod[] = $state([]);

	let modList: ModList;
	let maxCount: number = $state(20);
	let selectedMod: Mod | null = $state(null);

	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			mods = evt.payload;
		}).then((unlisten) => {
			unlistenFromQuery = unlisten;
		});

		return () => {
			unlistenFromQuery?.();
			api.thunderstore.stopQuerying();
		};
	});

	let hasRefreshed = $state(false);
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await api.thunderstore.query({ ...$modQuery, maxCount });
		if (selectedMod) {
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

	async function install(id: ModId) {
		await api.profile.install.mod(id);
		await refresh();
	}

	function onModClicked(evt: MouseEvent, mod: Mod) {
		if (evt.ctrlKey) {
			installLatest(mod);
		} else {
			modList.selectMod(mod);
		}
	}

	$effect(() => {
		if (maxCount > 0) {
			$modQuery;
			$activeProfile;
			refresh();
		}
	});

	let locked = $derived($activeProfileLocked);
</script>

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden pt-3 pl-3">
		<ModListFilters {sortOptions} queryArgs={modQuery} />

		{#if locked}
			<ProfileLockedBanner class="mr-4 mb-1" />
		{/if}

		<ModList
			{mods}
			queryArgs={modQuery}
			bind:this={modList}
			bind:maxCount
			bind:selected={selectedMod}
		>
			{#snippet placeholder()}
				{#if hasRefreshed}
					<div class="mt-4 text-lg">No matching mods found</div>
					<div class="text-primary-400">Try to adjust your search query/filters</div>
				{/if}
			{/snippet}

			{#snippet item({ mod, isSelected })}
				<ModListItem
					{mod}
					{isSelected}
					{contextItems}
					locked={$activeProfileLocked}
					oninstall={() => installLatest(mod)}
					onclick={(evt) => onModClicked(evt, mod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<ModDetails {locked} mod={selectedMod} {contextItems} onclose={() => (selectedMod = null)}>
			<InstallModButton mod={selectedMod} {install} {locked} />
		</ModDetails>
	{/if}
</div>
