<script lang="ts">
	import { invoke } from '$lib/invoke';
	import type { SortBy, Mod, ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';

	import ModList from '$lib/modlist/ModList.svelte';

	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { modQuery, activeProfileLocked, activeProfile } from '$lib/stores.svelte';
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ProfileLockedBanner from '$lib/modlist/ProfileLockedBanner.svelte';
	import ModDetails from '$lib/modlist/ModDetails.svelte';
	import ModListFilters from '$lib/modlist/ModListFilters.svelte';
	import { defaultContextItems } from '$lib/context';
	import InstallModButton from '$lib/modlist/InstallModButton.svelte';

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
			invoke('stop_querying_thunderstore');
		};
	});

	let hasRefreshed = $state(false);
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await invoke<Mod[]>('query_thunderstore', { args: { ...$modQuery, maxCount } });
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
		await invoke('install_mod', { modRef: id });
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
					<span class="text-lg">No matching mods found</span>
					<br />
					<span class="text-primary-400">Try to adjust your search query/filters</span>
				{/if}
			{/snippet}

			{#snippet item({ mod, isSelected })}
				<ModListItem
					{mod}
					{isSelected}
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
