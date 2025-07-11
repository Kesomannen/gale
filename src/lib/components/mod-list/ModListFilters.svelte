<script lang="ts">
	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import { selectItems } from '$lib/util';
	import { type SortBy, type QueryModsArgsWithoutMax } from '$lib/types';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { toSentenceCase } from 'js-convert-case';

	type Props = {
		sortOptions: SortBy[];
		queryArgs: QueryModsArgsWithoutMax;
	};

	let { sortOptions, queryArgs }: Props = $props();

	function getSelectedIncludes() {
		let selected = [];

		if (queryArgs.includeDeprecated) selected.push('Deprecated');
		if (queryArgs.includeNsfw) selected.push('NSFW');
		if (queryArgs.includeEnabled) selected.push('Enabled');
		if (queryArgs.includeDisabled) selected.push('Disabled');

		return selected;
	}
</script>

<div class="mb-1.5 flex flex-wrap gap-1.5 pr-3">
	<div class="relative flex-grow-3">
		<SearchBar bind:value={queryArgs.searchTerm} placeholder="Search for mods..." />
	</div>

	<div class="flex grow gap-1.5">
		<Select
			icon={queryArgs.sortOrder === 'descending' ? 'mdi:sort-descending' : 'mdi:sort-ascending'}
			triggerClass="grow basis-0 py-1.5"
			items={selectItems(['descending', 'ascending'], toSentenceCase)}
			bind:value={queryArgs.sortOrder}
			type="single"
		/>

		<Select
			icon="mdi:sort"
			triggerClass="grow basis-0 py-1.5"
			items={selectItems(sortOptions, toSentenceCase)}
			bind:value={queryArgs.sortBy}
			type="single"
		/>
	</div>
</div>

<div class="mb-1.5 flex items-start gap-1.5 pr-3">
	<ModListCategoryFilter
		label="Include categories"
		icon="mdi:filter"
		bind:selected={queryArgs.includeCategories}
		bind:excluded={queryArgs.excludeCategories}
	/>

	<ModListCategoryFilter
		label="Exclude categories"
		icon="mdi:filter-remove"
		bind:selected={queryArgs.excludeCategories}
		bind:excluded={queryArgs.includeCategories}
	/>

	<Select
		label="Include"
		icon="mdi:filter"
		triggerClass="min-w-36 grow basis-0 py-1.5"
		items={selectItems(['Deprecated', 'NSFW', 'Enabled', 'Disabled'])}
		onValueChange={(items) => {
			queryArgs.includeEnabled = items.includes('Enabled');
			queryArgs.includeDeprecated = items.includes('Deprecated');
			queryArgs.includeNsfw = items.includes('NSFW');
			queryArgs.includeDisabled = items.includes('Disabled');
		}}
		value={getSelectedIncludes()}
		type="multiple"
	/>
</div>
