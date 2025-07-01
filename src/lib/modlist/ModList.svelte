<script lang="ts">
	import ModDetails from '$lib/modlist/ModDetails.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { selectItems } from '$lib/util';
	import { type SortBy, type Mod, type QueryModsArgs, type ModContextItem } from '$lib/types';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';
	import { activeGame } from '$lib/stores.svelte';
	import Select from '$lib/components/Select.svelte';
	import type { Snippet } from 'svelte';
	import { toSentenceCase } from 'js-convert-case';

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

	type Props = {
		sortOptions: SortBy[];
		mods?: Mod[];
		maxCount?: number;
		queryArgs: Writable<QueryModsArgs>;
		selected: Mod | null;
		contextItems?: ModContextItem[];
		locked: boolean;
		banner?: Snippet;
		placeholder?: Snippet;
		item?: Snippet<
			[{ mod: Mod; index: number; contextItems: ModContextItem[]; isSelected: boolean }]
		>;
		details?: Snippet;
	};

	let {
		sortOptions,
		mods = $bindable([]),
		maxCount = $bindable(20),
		queryArgs,
		selected = $bindable(),
		contextItems = [],
		locked,
		banner,
		placeholder,
		item,
		details
	}: Props = $props();

	let allContextItems = $derived([...contextItems, ...defaultContextItems]);

	let listStart = $state(0);
	let listEnd = $state(0);
	let virtualList: VirtualList<Mod>;

	$effect(() => {
		if (listEnd > mods.length - 2 && mods.length === maxCount) {
			maxCount += 20;
		}
	});

	$effect(() => {
		$queryArgs;
		virtualList?.scrollTo(0);
	});

	$effect(() => {
		$activeGame;
		selected = null;
	});

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

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden pt-3 pl-3">
		<div class="mb-1.5 flex flex-wrap gap-1.5 pr-3">
			<div class="relative flex-grow-3">
				<SearchBar bind:value={$queryArgs.searchTerm} placeholder="Search for mods..." />
			</div>

			<div class="flex grow gap-1.5">
				<Select
					icon={$queryArgs.sortOrder === 'descending'
						? 'mdi:sort-descending'
						: 'mdi:sort-ascending'}
					triggerClass="grow basis-0 py-1.5"
					items={selectItems(['descending', 'ascending'], toSentenceCase)}
					bind:value={$queryArgs.sortOrder}
					type="single"
				/>

				<Select
					icon="mdi:sort"
					triggerClass="grow basis-0 py-1.5"
					items={selectItems(sortOptions, toSentenceCase)}
					bind:value={$queryArgs.sortBy}
					type="single"
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

			<Select
				label="Include"
				icon="mdi:filter"
				triggerClass="min-w-36 grow basis-0 py-1.5"
				items={selectItems(['Deprecated', 'NSFW', 'Enabled', 'Disabled'])}
				onValueChange={(items) => {
					$queryArgs.includeEnabled = items.includes('Enabled');
					$queryArgs.includeDeprecated = items.includes('Deprecated');
					$queryArgs.includeNsfw = items.includes('NSFW');
					$queryArgs.includeDisabled = items.includes('Disabled');
				}}
				value={getSelectedIncludes()}
				type="multiple"
			/>
		</div>

		{@render banner?.()}

		{#if mods.length === 0}
			<div class="text-primary-300 mt-4 text-center">
				{@render placeholder?.()}
			</div>
		{:else}
			<VirtualList
				itemHeight={66}
				items={mods}
				bind:this={virtualList}
				bind:start={listStart}
				bind:end={listEnd}
			>
				{#snippet children({ item: mod, index })}
					{@render item?.({
						mod,
						index,
						contextItems: allContextItems,
						isSelected: selected?.uuid === mod.uuid
					})}
				{/snippet}
			</VirtualList>
		{/if}
	</div>

	{#if selected !== null}
		<ModDetails
			{locked}
			mod={selected}
			contextItems={allContextItems}
			onclose={() => (selected = null)}
		>
			{@render details?.()}
		</ModDetails>
	{/if}
</div>
