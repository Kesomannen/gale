<script lang="ts">
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VirtualList from './VirtualList.svelte';

	import { sentenceCase, isBefore } from '$lib/util';
	import { SortBy, type Mod, type QueryModsArgs, SortOrder } from '$lib/models';

	import Icon from '@iconify/svelte';
	import { Button, Select } from 'bits-ui';

	import { categories } from '$lib/stores';
	import type { Writable } from 'svelte/store';

	export let sortOptions: SortBy[];

	export let mods: Mod[] = [];
	export let activeMod: Mod | undefined;
	export let queryArgs: Writable<QueryModsArgs>;

	let listStart = 0;
	let listEnd = 0;
	let virtualList: VirtualList<Mod>;

	let increasedCount = false;

	$: if (listEnd > mods.length - 2 && mods.length === $queryArgs.maxCount) {
		increasedCount = true;
		$queryArgs.maxCount += 20;
	}

	$: {
		// scroll to top when query changes, except for the max count
		$queryArgs;
		if (increasedCount) {
			increasedCount = false;
		} else {
			virtualList?.scrollTo(0);
		}
	}

	function onModClicked(mod: Mod) {
		if (activeMod === undefined || activeMod.uuid !== mod.uuid) {
			activeMod = mod;
		} else {
			activeMod = undefined;
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
		<div class="mb-1.5 flex gap-1.5 pr-3">
			<div class="relative flex-grow">
				<SearchBar bind:value={$queryArgs.searchTerm} placeholder="Search for mods..." />
			</div>

			<Dropdown
				items={[SortOrder.Descending, SortOrder.Ascending]}
				bind:selected={$queryArgs.sortOrder}
				getLabel={sentenceCase}
			>
				<Select.Trigger
					let:text
					let:open
					slot="trigger"
					class="flex w-48 items-center gap-2 rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 text-slate-300 hover:border-opacity-100"
				>
					<Icon
						class="text-lg text-slate-400"
						icon={$queryArgs.sortOrder === SortOrder.Descending
							? 'mdi:sort-descending'
							: 'mdi:sort-ascending'}
					/>
					<span class="flex-shrink truncate">{text}</span>
					<Icon
						class="ase-out ml-auto origin-center transform text-xl text-slate-400 transition-all duration-100 {open
							? 'rotate-180'
							: 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown items={sortOptions} bind:selected={$queryArgs.sortBy} getLabel={sentenceCase}>
				<Select.Trigger
					let:text
					let:open
					slot="trigger"
					class="flex w-48 items-center gap-2 rounded-lg border border-gray-500 border-opacity-0
								bg-gray-900 px-3 text-slate-300 hover:border-opacity-100"
				>
					<Icon class="text-lg text-slate-400" icon="mdi:sort" />
					<span class="flex-shr truncate">{text}</span>
					<Icon
						class="ml-auto origin-center transform text-xl text-slate-400 transition-all duration-100 ease-out {open
							? 'rotate-180'
							: 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		</div>

		<div class="mb-1.5 flex items-start gap-1.5 pr-3">
			<Dropdown
				items={$categories
					.map((category) => category.name)
					.filter((category) => !$queryArgs.excludeCategories.includes(category))
					.toSorted()}
				multiple={true}
				bind:selected={$queryArgs.includeCategories}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex w-1/2 items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 py-1.5 hover:border-opacity-100"
				>
					<Icon class="mr-2 flex-shrink-0 text-lg text-slate-400" icon="mdi:filter" />
					{#if $queryArgs.includeCategories.length === 0}
						<span class="truncate text-slate-300">Include categories</span>
					{:else}
						<div class="flex flex-wrap gap-1 overflow-hidden">
							{#each $queryArgs.includeCategories as category}
								<div
									class="overflow-hidden rounded-lg bg-gray-800 py-0.5 pl-2 pr-0.5 text-sm text-slate-200"
								>
									<span class="overflow-hidden truncate">{category}</span>

									<Button.Root
										class="ml-0.5 rounded-lg px-1.5 hover:bg-gray-700"
										on:click={(evt) => {
											evt.stopPropagation();
											$queryArgs.includeCategories = $queryArgs.includeCategories.filter(
												(cat) => cat !== category
											);
										}}
									>
										x
									</Button.Root>
								</div>
							{/each}
						</div>
					{/if}
					<Icon
						class="ml-auto flex-shrink-0 origin-center transform text-xl text-slate-400 transition-all
										duration-100 ease-out {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown
				items={$categories
					.map((category) => category.name)
					.filter((category) => !$queryArgs.includeCategories.includes(category))
					.toSorted()}
				multiple={true}
				bind:selected={$queryArgs.excludeCategories}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex w-1/2 items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0
							bg-gray-900 px-3 py-1.5 hover:border-opacity-100"
				>
					<Icon class="mr-2 flex-shrink-0 text-lg text-slate-400" icon="mdi:filter-remove" />
					{#if $queryArgs.excludeCategories.length === 0}
						<span class="truncate text-slate-300">Exclude categories</span>
					{:else}
						<div class="mr-2 flex flex-wrap gap-1">
							{#each $queryArgs.excludeCategories as category}
								<div
									class="overflow-hidden rounded-lg bg-gray-800 py-0.5 pl-2 pr-0.5 text-sm text-slate-200"
								>
									<span class="overflow-hidden truncate">{category}</span>

									<Button.Root
										class="ml-0.5 rounded-lg px-1.5 hover:bg-gray-700"
										on:click={(evt) => {
											evt.stopPropagation();
											$queryArgs.excludeCategories = $queryArgs.excludeCategories.filter(
												(cat) => cat !== category
											);
										}}
									>
										x
									</Button.Root>
								</div>
							{/each}
						</div>
					{/if}
					<Icon
						class="ml-auto flex-shrink-0 origin-center transform text-xl text-slate-400 transition-all duration-100 ease-out {open
							? 'rotate-180'
							: 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown
				items={['Deprecated', 'NSFW', 'Enabled', 'Disabled']}
				selected={getSelectedIncludes()}
				multiple={true}
				onSelectedChange={(items) => {
					$queryArgs.includeEnabled = items.includes('Enabled');
					$queryArgs.includeDeprecated = items.includes('Deprecated');
					$queryArgs.includeNsfw = items.includes('NSFW');
					$queryArgs.includeDisabled = items.includes('Disabled');
				}}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex items-center rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 py-1.5
								pl-3 pr-2 text-slate-300 hover:border-opacity-100"
				>
					<Icon class="mr-2 flex-shrink-0 text-lg text-slate-400" icon="mdi:filter-variant" />
					Include
					<Icon
						class="ml-6 flex-shrink-0 origin-center transform text-xl text-slate-400 transition-all
										duration-100 ease-out {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		</div>

		<slot name="banner" />

		{#if mods.length === 0}
			<div class="mt-4 text-center text-lg text-slate-300">No mods found 😥</div>
		{:else}
			<VirtualList
				bind:this={virtualList}
				itemHeight={66}
				items={mods}
				let:item={mod}
				bind:start={listStart}
				bind:end={listEnd}
			>
				<button class="contents" on:click={() => onModClicked(mod)}>
					<slot name="item" {mod} isSelected={activeMod === mod} />
				</button>
			</VirtualList>
		{/if}
	</div>

	{#if activeMod}
		<ModDetailsMenu mod={activeMod} onClose={() => (activeMod = undefined)}>
			<slot name="details" />
			<svelte:fragment slot="dropdown">
				<slot name="dropdown" />
			</svelte:fragment>
		</ModDetailsMenu>
	{/if}
</div>
