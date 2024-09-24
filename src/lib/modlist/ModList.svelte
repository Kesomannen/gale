<script lang="ts">
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import { sentenceCase, isBefore } from '$lib/util';
	import { SortBy, type Mod, type QueryModsArgs, SortOrder } from '$lib/models';

	import Icon from '@iconify/svelte';
	import { Button, Select } from 'bits-ui';

	import VirtualList from '@sveltejs/svelte-virtual-list';
	import { categories } from '$lib/stores';
	import type { Writable } from 'svelte/store';
	import { createEventDispatcher } from 'svelte';

	export let sortOptions: SortBy[];

	export let mods: Mod[] = [];
	export let activeMod: Mod | undefined;
	export let queryArgs: Writable<QueryModsArgs>;
	export let reorderable = false;
	export let showInstalledIcon = false;

	let listStart = 0;
	let listEnd = 0;

	let maxCount = 20;
	let searchTerm = $queryArgs.searchTerm;
	let includeCategories = $queryArgs.includeCategories;
	let excludeCategories = $queryArgs.excludeCategories;
	let includeNsfw = $queryArgs.includeNsfw;
	let includeEnabled = $queryArgs.includeEnabled;
	let includeDeprecated = $queryArgs.includeDeprecated;
	let includeDisabled = $queryArgs.includeDisabled;
	let sortBy = $queryArgs.sortBy;
	let sortOrder = $queryArgs.sortOrder;

	const dispatch = createEventDispatcher<{
		reorder: {
			uuid: string;
			delta: number;
		};
		finishReorder: {
			uuid: string;
			totalDelta: number;
		};
		onModCtrlClicked: {
			mod: Mod;
		};
	}>();

	$: {
		$queryArgs = {
			maxCount,
			searchTerm,
			includeCategories,
			excludeCategories,
			includeNsfw,
			includeEnabled,
			includeDeprecated,
			includeDisabled,
			sortBy,
			sortOrder
		};
	}

	$: if (listEnd > mods.length - 2 && mods.length === maxCount) {
		maxCount += 20;
	}

	function onModClicked(mod: Mod, evt: MouseEvent) {
		if (evt.ctrlKey) {
			dispatch('onModCtrlClicked', { mod });
			return;
		}

		if (activeMod === undefined || activeMod.uuid !== mod.uuid) {
			activeMod = mod;
		} else {
			activeMod = undefined;
		}
	}

	function getSelectedIncludes() {
		let selected = [];

		if (includeDeprecated) selected.push('Deprecated');
		if (includeNsfw) selected.push('NSFW');
		if (includeEnabled) selected.push('Enabled');
		if (includeDisabled) selected.push('Disabled');

		return selected;
	}

	let dragElement: HTMLElement | null;
	let totalDelta = 0;

	function onDragStart(evt: DragEvent) {
		if (!reorderable) return;
		if (!evt.dataTransfer) return;

		totalDelta = 0;
		dragElement = evt.currentTarget as HTMLElement;
		evt.dataTransfer.effectAllowed = 'move';
		evt.dataTransfer.setData('text/html', dragElement.outerHTML);
	}

	function onDragOver(evt: DragEvent) {
		if (!reorderable || !dragElement) return;

		let target = evt.currentTarget as HTMLElement;
		let draggingUuid = dragElement.dataset.uuid;
		let targetUuid = target.dataset.uuid;

		if (draggingUuid === targetUuid) return;

		if (isBefore(dragElement, target)) {
			dispatch('reorder', { uuid: draggingUuid!, delta: -1 });
			totalDelta--;
		} else {
			dispatch('reorder', { uuid: draggingUuid!, delta: 1 });
			totalDelta++;
		}

		dragElement = target;
	}

	function onDragEnd(evt: DragEvent) {
		if (!reorderable || !dragElement) return;

		let uuid = dragElement.dataset.uuid!;

		dispatch('finishReorder', { uuid, totalDelta });
		dragElement = null;
		totalDelta = 0;
	}

	const dragScrollSpeed = 10;
	const dragScrollArea = 100;

	function onDrag(evt: DragEvent) {
		/*
		if (window.innerHeight - evt.clientY < dragScrollArea) {
			viewport.scrollBy(0, dragScrollSpeed);
		} else if (evt.clientY < dragScrollArea) {
			viewport.scrollBy(0, -dragScrollSpeed);
		}
		*/
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex w-[60%] flex-grow flex-col overflow-hidden pl-3 pt-3">
		<div class="mb-1.5 flex gap-1.5 pr-3">
			<div class="relative flex-grow">
				<SearchBar bind:value={searchTerm} placeholder="Search for mods..." />
			</div>

			<Dropdown
				items={[SortOrder.Descending, SortOrder.Ascending]}
				bind:selected={sortOrder}
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
						icon={sortOrder === SortOrder.Descending ? 'mdi:sort-descending' : 'mdi:sort-ascending'}
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

			<Dropdown items={sortOptions} bind:selected={sortBy} getLabel={sentenceCase}>
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
					.filter((category) => !excludeCategories.includes(category))
					.toSorted()}
				multiple={true}
				bind:selected={includeCategories}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex w-1/2 items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 py-1.5 hover:border-opacity-100"
				>
					<Icon class="mr-2 flex-shrink-0 text-lg text-slate-400" icon="mdi:filter" />
					{#if includeCategories.length === 0}
						<span class="truncate text-slate-300">Include categories</span>
					{:else}
						<div class="flex flex-wrap gap-1 overflow-hidden">
							{#each includeCategories as category}
								<div
									class="overflow-hidden rounded-lg bg-gray-800 py-0.5 pl-2 pr-0.5 text-sm text-slate-200"
								>
									<span class="overflow-hidden truncate">{category}</span>

									<Button.Root
										class="ml-0.5 rounded-lg px-1.5 hover:bg-gray-700"
										on:click={(evt) => {
											evt.stopPropagation();
											includeCategories = includeCategories.filter((cat) => cat !== category);
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
					.filter((category) => !includeCategories.includes(category))
					.toSorted()}
				multiple={true}
				bind:selected={excludeCategories}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex w-1/2 items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0
							bg-gray-900 px-3 py-1.5 hover:border-opacity-100"
				>
					<Icon class="mr-2 flex-shrink-0 text-lg text-slate-400" icon="mdi:filter-remove" />
					{#if excludeCategories.length === 0}
						<span class="truncate text-slate-300">Exclude categories</span>
					{:else}
						<div class="mr-2 flex flex-wrap gap-1">
							{#each excludeCategories as category}
								<div
									class="overflow-hidden rounded-lg bg-gray-800 py-0.5 pl-2 pr-0.5 text-sm text-slate-200"
								>
									<span class="overflow-hidden truncate">{category}</span>

									<Button.Root
										class="ml-0.5 rounded-lg px-1.5 hover:bg-gray-700"
										on:click={(evt) => {
											evt.stopPropagation();
											excludeCategories = excludeCategories.filter((cat) => cat !== category);
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
					includeEnabled = items.includes('Enabled');
					includeDeprecated = items.includes('Deprecated');
					includeNsfw = items.includes('NSFW');
					includeDisabled = items.includes('Disabled');
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
				itemHeight={66}
				items={mods}
				let:item={mod}
				bind:start={listStart}
				bind:end={listEnd}
			>
				<ModListItem
					on:click={(evt) => onModClicked(mod, evt)}
					on:dragstart={onDragStart}
					on:dragover={onDragOver}
					on:dragend={onDragEnd}
					on:drag={onDrag}
					isInstalled={mod.isInstalled}
					draggable={reorderable}
					isSelected={activeMod?.uuid == mod.uuid}
					{showInstalledIcon}
					{mod}
				>
					<slot name="item" {mod} />
				</ModListItem>
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
