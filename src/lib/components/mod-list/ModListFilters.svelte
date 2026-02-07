<script lang="ts">
	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import { selectItems } from '$lib/util';
	import { type SortBy, type QueryModsArgsWithoutMax } from '$lib/types';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { toSentenceCase } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		sortOptions: SortBy[];
		queryArgs: QueryModsArgsWithoutMax;
	};

	let { sortOptions, queryArgs }: Props = $props();

	function getSelectedIncludes() {
		let selected = [];

		if (queryArgs.includeDeprecated) selected.push('deprecated');
		if (queryArgs.includeNsfw) selected.push('NSFW');
		if (queryArgs.includeEnabled) selected.push('enabled');
		if (queryArgs.includeDisabled) selected.push('disabled');

		return selected;
	}

	const optionsTranslate: Record<string, string> = {
		deprecated: m.modListFilters_options_deprecated(),
		NSFW: m.modListFilters_options_NSFW(),
		enabled: m.modListFilters_options_enabled(),
		disabled: m.modListFilters_options_disabled(),
		ascending: m.modListFilters_options_ascending(),
		descending: m.modListFilters_options_descending(),
		lastUpdated: m.modListFilters_options_lastUpdated(),
		newest: m.modListFilters_options_newest(),
		rating: m.modListFilters_options_rating(),
		downloads: m.modListFilters_options_downloads(),
		custom: m.modListFilters_options_custom(),
		installDate: m.modListFilters_options_installDate(),
		diskSpace: m.modListFilters_options_diskSpace(),
		name: m.modListFilters_options_name(),
		author: m.modListFilters_options_author()
	};

	function getOptionsLabel(item: string): string {
		return optionsTranslate[item] ?? toSentenceCase(item);
	}
</script>

<div class="mb-1.5 flex flex-wrap gap-1.5 pr-3">
	<div class="relative flex-grow-3">
		<SearchBar
			bind:value={queryArgs.searchTerm}
			placeholder={m.modListFilters_searchBar_placeholder()}
		/>
	</div>

	<div class="flex grow gap-1.5">
		<Select
			icon={queryArgs.sortOrder === 'descending' ? 'mdi:sort-descending' : 'mdi:sort-ascending'}
			triggerClass="grow basis-0 py-1.5"
			items={selectItems(['descending', 'ascending'], getOptionsLabel)}
			bind:value={queryArgs.sortOrder}
			type="single"
		/>

		<Select
			icon="mdi:sort"
			triggerClass="grow basis-0 py-1.5"
			items={selectItems(sortOptions, getOptionsLabel)}
			bind:value={queryArgs.sortBy}
			type="single"
		/>
	</div>
</div>

<div class="mb-1.5 flex items-start gap-1.5 pr-3">
	<ModListCategoryFilter
		label={m.modListFilters_filter_include()}
		icon="mdi:filter"
		bind:selected={queryArgs.includeCategories}
		bind:excluded={queryArgs.excludeCategories}
	/>

	<ModListCategoryFilter
		label={m.modListFilters_filter_exclude()}
		icon="mdi:filter-remove"
		bind:selected={queryArgs.excludeCategories}
		bind:excluded={queryArgs.includeCategories}
	/>

	<Select
		label={m.modListFilters_select_title()}
		icon="mdi:filter"
		triggerClass="min-w-36 grow basis-0 py-1.5"
		items={selectItems(['deprecated', 'NSFW', 'enabled', 'disabled'], getOptionsLabel)}
		onValueChange={(items) => {
			queryArgs.includeEnabled = items.includes('enabled');
			queryArgs.includeDeprecated = items.includes('deprecated');
			queryArgs.includeNsfw = items.includes('NSFW');
			queryArgs.includeDisabled = items.includes('disabled');
		}}
		value={getSelectedIncludes()}
		type="multiple"
	/>
</div>
