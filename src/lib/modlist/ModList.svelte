<script lang="ts">
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VirtualList from './VirtualList.svelte';

	import { sentenceCase } from '$lib/util';
	import { SortBy, type Mod, type QueryModsArgs, SortOrder } from '$lib/models';

	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';

	export let sortOptions: SortBy[];

	export let mods: Mod[] = [];
	export let activeMod: Mod | null;
	export let queryArgs: Writable<QueryModsArgs>;

	let listStart = 0;
	let listEnd = 0;
	let virtualList: VirtualList<Mod>;

	let increasedCount = false;

	$queryArgs.maxCount = 30;

	$: if (listEnd > mods.length - 2 && mods.length === $queryArgs.maxCount) {
		increasedCount = true;
		$queryArgs.maxCount += 30;
	}

	$: {
		// scroll to top when query changes except for the max count
		$queryArgs;
		if (increasedCount) {
			increasedCount = false;
		} else {
			virtualList?.scrollTo(0);
		}
	}

	function onModClicked(mod: Mod) {
		if (activeMod === null || activeMod.uuid !== mod.uuid) {
			activeMod = mod;
		} else {
			activeMod = null;
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
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex w-[60%] flex-grow flex-col overflow-hidden pl-3 pt-3">
		<div class="mb-1.5 flex flex-wrap gap-1.5 pr-3">
			<div class="relative flex-grow-[3]">
				<SearchBar bind:value={$queryArgs.searchTerm} placeholder="Search for mods..." />
			</div>

			<div class="flex flex-grow gap-1.5">
				<Dropdown
					class="flex-grow py-1.5"
					icon={$queryArgs.sortOrder === SortOrder.Descending
						? 'mdi:sort-descending'
						: 'mdi:sort-ascending'}
					items={[SortOrder.Descending, SortOrder.Ascending]}
					bind:selected={$queryArgs.sortOrder}
					getLabel={sentenceCase}
				/>

				<Dropdown
					class="flex-grow py-1.5"
					items={sortOptions}
					bind:selected={$queryArgs.sortBy}
					getLabel={sentenceCase}
					icon="mdi:sort"
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
				class="min-w-36 flex-grow py-1.5"
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
				bind:this={virtualList}
				itemHeight={66}
				items={mods}
				bind:start={listStart}
				bind:end={listEnd}
				let:item={mod}
				let:index
			>
				<button class="contents" on:click={() => onModClicked(mod)}>
					<slot name="item" {mod} {index} isSelected={activeMod === mod} />
				</button>
			</VirtualList>
		{/if}
	</div>

	{#if activeMod !== null}
		<ModDetailsMenu mod={activeMod} onClose={() => (activeMod = null)}>
			<slot name="details" />
			<svelte:fragment slot="context">
				<slot name="context" />
			</svelte:fragment>
		</ModDetailsMenu>
	{/if}
</div>
