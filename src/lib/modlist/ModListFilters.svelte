<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { type SortBy, type QueryModsArgs, SortOrder } from '$lib/models';
	import { sentenceCase } from '$lib/util';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';

	export let sortOptions: SortBy[];
	export let queryArgs: Writable<QueryModsArgs>;

	function getSelectedIncludes() {
		let selected = [];

		if ($queryArgs.includeDeprecated) selected.push('Deprecated');
		if ($queryArgs.includeNsfw) selected.push('NSFW');
		if ($queryArgs.includeEnabled) selected.push('Enabled');
		if ($queryArgs.includeDisabled) selected.push('Disabled');

		return selected;
	}
</script>

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
