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
	<div class="flex flex-col flex-grow w-[60%] pt-3 pl-3 overflow-hidden">
		<div class="flex gap-1.5 mb-1.5 pr-3">
			<div class="flex-grow relative">
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
					class="flex items-center gap-2 w-48 bg-gray-900 rounded-lg px-3 text-slate-300
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon
						class="text-slate-400 text-lg"
						icon={sortOrder === SortOrder.Descending ? 'mdi:sort-descending' : 'mdi:sort-ascending'}
					/>
					<span class="flex-shrink truncate">{text}</span>
					<Icon
						class="text-slate-400 text-xl transition-all duration-100 ease-out ml-auto
										transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown items={sortOptions} bind:selected={sortBy} getLabel={sentenceCase}>
				<Select.Trigger
					let:text
					let:open
					slot="trigger"
					class="flex items-center gap-2 w-48 bg-gray-900 rounded-lg px-3 text-slate-300
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-lg" icon="mdi:sort" />
					<span class="flex-shr truncate">{text}</span>
					<Icon
						class="text-slate-400 text-xl transition-all duration-100 ease-out ml-auto
										transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		</div>

		<div class="flex gap-1.5 mb-1.5 pr-3">
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
					class="flex items-center w-1/2 bg-gray-900 rounded-lg px-3 py-1.5 overflow-hidden
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-lg mr-2 flex-shrink-0" icon="mdi:filter" />
					{#if includeCategories.length === 0}
						<span class="text-slate-300 truncate">Include categories</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each includeCategories as category}
								<div class="bg-blue-600 text-white rounded-full pl-3 pr-0.5 py-0.5 text-sm">
									<span class="truncate overflow-hidden">{category}</span>

									<Button.Root
										class="px-1.5 rounded-full hover:bg-blue-500"
										on:click={(evt) => {
											evt.stopPropagation();
											includeCategories = includeCategories.filter((c) => c !== category);
										}}
									>
										x
									</Button.Root>
								</div>
							{/each}
						</div>
					{/if}
					<Icon
						class="text-slate-400 text-xl transition-all duration-100 ease-out ml-auto flex-shrink-0
										transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
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
					class="flex items-center w-1/2 bg-gray-900 rounded-lg px-3 py-1.5 overflow-hidden
							border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-lg mr-2 flex-shrink-0" icon="mdi:filter-remove" />
					{#if excludeCategories.length === 0}
						<span class="text-slate-300 truncate">Exclude categories</span>
					{:else}
						<div class="flex flex-wrap gap-1 mr-2">
							{#each excludeCategories as category}
								<div
									class="bg-red-600 text-white rounded-xl pl-3 pr-0.5 py-0.5 text-sm text-left align-middle"
								>
									{category}

									<Button.Root
										class="px-1.5 rounded-full hover:bg-red-500"
										on:click={(evt) => {
											evt.stopPropagation();
											excludeCategories = excludeCategories.filter((c) => c !== category);
										}}
									>
										x
									</Button.Root>
								</div>
							{/each}
						</div>
					{/if}
					<Icon
						class="text-slate-400 text-xl transition-all duration-100 ease-out ml-auto flex-shrink-0
									transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown
				items={['Deprecated', 'NSFW', 'Disabled']}
				selected={getSelectedIncludes()}
				multiple={true}
				onSelectedChange={(items) => {
					includeDeprecated = items.includes('Deprecated');
					includeNsfw = items.includes('NSFW');
					includeDisabled = items.includes('Disabled');
				}}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex items-center bg-gray-900 text-slate-300 rounded-lg pl-3 pr-2 py-1
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-lg flex-shrink-0 mr-2" icon="mdi:filter-variant" />
					Include
					<Icon
						class="text-slate-400 text-xl transition-all ml-6 flex-shrink-0 duration-100 ease-out
										transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		</div>

		<slot name="banner" />

		{#if mods.length === 0}
			<div class="text-slate-300 text-lg text-center mt-4">No mods found 😥</div>
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
					let:isInstalled
					draggable={reorderable}
					isSelected={activeMod?.uuid == mod.uuid}
					{mod}
				>
					<slot name="item" {mod} {isInstalled} />
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
