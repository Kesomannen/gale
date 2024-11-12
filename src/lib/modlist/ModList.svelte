<script lang="ts">
	import ModDetails from '$lib/modlist/ModDetails.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { communityUrl, sentenceCase } from '$lib/util';
	import {
		SortBy,
		type Mod,
		type QueryModsArgs,
		SortOrder,
		type ModContextItem
	} from '$lib/models';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';
	import { activeGame } from '$lib/stores';

	const defaultContextItems: ModContextItem[] = [
		{
			label: 'Open website',
			icon: 'mdi:open-in-new',
			onclick: (mod) => openIfNotNull(mod.websiteUrl),
			showFor: (mod) => mod.websiteUrl !== null && mod.websiteUrl.length > 0
		},
		{
			label: 'Donate',
			icon: 'mdi:heart',
			onclick: (mod) => openIfNotNull(mod.donateUrl),
			showFor: (mod) => mod.donateUrl !== null
		}
	];

	export let sortOptions: SortBy[];

	export let mods: Mod[] = [];
	export let maxCount = 20;
	export let queryArgs: Writable<QueryModsArgs>;

	export let selected: Mod | null;
	export let contextItems: ModContextItem[] = [];

	$: allContextItems = [...contextItems, ...defaultContextItems];

	let listStart = 0;
	let listEnd = 0;
	let virtualList: VirtualList<Mod>;

	$: if (listEnd > mods.length - 2 && mods.length === maxCount) {
		maxCount += 20;
		console.log('increasing max count');
	}

	$: {
		$queryArgs;
		virtualList?.scrollTo(0);
	}

	$: {
		$activeGame;
		selected = null;
	}

	export function selectMod(mod: Mod) {
		if (selected === null || selected.uuid !== mod.uuid) {
			selected = mod;
		} else {
			selected = null;
		}
	}

	function getSelectedIncludes() {
		let selected = [];

		if ($queryArgs.includeDeprecated) selected.push('Deprecated');
		if ($queryArgs.includeNsfw) selected.push('NSFW');
		if ($queryArgs.includeEnabled) selected.push('Enabled');
		if ($queryArgs.includeDisabled) selected.push('Disabled');

		return selected;
	}

	function openIfNotNull(url: string | null) {
		if (url !== null) open(url);
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex w-[60%] flex-grow flex-col overflow-hidden pl-3 pt-3">
		<div class="mb-1.5 flex flex-wrap gap-1.5 pr-3">
			<div class="relative flex-grow-[3]">
				<SearchBar bind:value={$queryArgs.searchTerm} placeholder="Search for mods..." />
			</div>

			<div class="flex flex-grow gap-1.5">
				<Dropdown
					class="flex-grow basis-0 py-1.5"
					icon={$queryArgs.sortOrder === SortOrder.Descending
						? 'mdi:sort-descending'
						: 'mdi:sort-ascending'}
					items={[SortOrder.Descending, SortOrder.Ascending]}
					bind:selected={$queryArgs.sortOrder}
					getLabel={sentenceCase}
					multiple={false}
				/>

				<Dropdown
					class="flex-grow basis-0 py-1.5"
					items={sortOptions}
					bind:selected={$queryArgs.sortBy}
					getLabel={sentenceCase}
					icon="mdi:sort"
					multiple={false}
				/>
			</div>
		</div>

		<div class="mb-1.5 flex items-start gap-1.5 pr-3">
			<ModListCategoryFilter
				label="Include categories"
				icon="mdi:filter"
				bind:selected={$queryArgs.includeCategories}
				bind:excluded={$queryArgs.excludeCategories}
			/>

			<ModListCategoryFilter
				label="Exclude categories"
				icon="mdi:filter-remove"
				bind:selected={$queryArgs.excludeCategories}
				bind:excluded={$queryArgs.includeCategories}
			/>

			<Dropdown
				overrideLabel="Include"
				icon="mdi:filter"
				class="min-w-36 flex-grow basis-0 py-1.5"
				items={['Deprecated', 'NSFW', 'Enabled', 'Disabled']}
				selected={getSelectedIncludes()}
				onSelectedChange={(items) => {
					$queryArgs.includeEnabled = items.includes('Enabled');
					$queryArgs.includeDeprecated = items.includes('Deprecated');
					$queryArgs.includeNsfw = items.includes('NSFW');
					$queryArgs.includeDisabled = items.includes('Disabled');
				}}
				multiple
			/>
		</div>

		<slot name="banner" />

		{#if mods.length === 0}
			<div class="mt-4 text-center text-lg text-slate-300">No mods found 😥</div>
		{:else}
			<VirtualList
				itemHeight={66}
				items={mods}
				bind:this={virtualList}
				bind:start={listStart}
				bind:end={listEnd}
				let:item={mod}
				let:index
			>
				<slot
					name="item"
					data={{
						mod,
						index,
						contextItems: allContextItems,
						isSelected: selected?.uuid === mod.uuid
					}}
				/>
			</VirtualList>
		{/if}
	</div>

	{#if selected !== null}
		<ModDetails mod={selected} contextItems={allContextItems} on:close={() => (selected = null)}>
			<slot name="details" />
		</ModDetails>
	{/if}
</div>
