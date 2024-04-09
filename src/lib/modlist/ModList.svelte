<script lang="ts">
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import ModListSortOption from '$lib/modlist/ModListSortOption.svelte';

	import { SortBy, type Mod, type QueryModsArgs, type DropdownOption } from '$lib/models';

	import Icon from '@iconify/svelte';
	import { Button, Popover, Select, Separator } from 'bits-ui';

	import { slide } from 'svelte/transition';
	import { quadOut } from 'svelte/easing';

	interface SortOption {
		value: SortBy;
		label: string;
	}

	const pageSize = 20;
	const sortOptions: SortOption[] = [
		{ value: SortBy.LastUpdated, label: 'Last updated' },
		{ value: SortBy.Rating, label: 'Rating' },
		{ value: SortBy.Downloads, label: 'Downloads' },
	];

	const allCategories = [
		'Mods',
		'Modpacks',
		'Interiors',
		'Audio',
		'Tools',
		'Moons',
		'Client-side',
		'Server-side',
		'Emotes',
		'Cosmetics',
		'TV Videos'
	];

	let page = 0;

	let searchTerm: string | undefined;
	let categories: string[] = [];
	let includeNsfw = false;
	let includeDeprecated = false;
	let sortBy: SortOption = sortOptions[0];

	export let mods: Mod[] = [];
	export let activeMod: Mod | undefined = undefined;
	export let extraDropdownOptions: DropdownOption[] = [];
	export let queryArgs: QueryModsArgs;

	$: queryArgs = {
		page,
		pageSize,
		searchTerm,
		categories,
		includeNsfw,
		includeDeprecated,
		descending: true,
		sortBy: sortBy.value
	};

	$: {
		if (mods.length === 0 && page > 0) {
			page--;
		}
	}

	function onModClicked(mod: Mod) {
		if (activeMod === undefined || activeMod.uuid !== mod.uuid) {
			activeMod = mod;
		} else {
			activeMod = undefined;
		}
	}

	function onFilterChange(category: string, value: boolean) {
		if (value) {
			categories = [...categories, category];
		} else {
			categories = categories.filter((c) => c !== category);
		}
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex flex-col flex-grow w-[60%] pt-3 pl-3 overflow-hidden">
		<div class="flex gap-2 mb-2 pr-3">
			<div class="relative flex-grow">
				<input
					type="text"
					class="w-full py-2 pr-10 pl-12 rounded-lg bg-gray-900 text-slate-300 truncate"
					bind:value={searchTerm}
					placeholder="Search for mods..."
				/>
				<Icon class="absolute left-[12px] top-[9px] text-slate-400 text-2xl" icon="mdi:magnify" />
			</div>

			<Select.Root items={sortOptions} bind:selected={sortBy}>
				<Select.Trigger
					class="flex items-center flex-shrink-0 w-48 bg-gray-900 rounded-lg px-3 
                                       border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-2xl mr-2" icon="mdi:sort" />
					<Select.Value class="text-slate-300 text-left w-full" />
					<Icon class="text-slate-400 text-2xl ml-auto" icon="mdi:chevron-down" />
				</Select.Trigger>
				<Select.Content
					class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-48 rounded-lg border border-gray-600"
					transition={slide}
					transitionConfig={{ duration: 100 }}
				>
					{#each sortOptions as item}
						<Select.Item
							value={item.value}
							class="flex items-center px-3 py-1 text-slate-400 hover:text-slate-200 text-left rounded-lg hover:bg-gray-700 cursor-default"
						>
							{item.label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<div class="flex gap-2 mb-2 pr-3">
			<div class="flex items-center flex-shrink-0">
				<Button.Root
					class="p-1 hover:bg-gray-700 disabled:bg-opacity-0 disabled:cursor-not-allowed rounded-lg transition-all group"
					on:click={() => page--}
					disabled={page === 0}
				>
					<Icon
						class="text-white text-2xl group-disabled:text-slate-400 align-middle"
						icon="mdi:chevron-left"
					/>
				</Button.Root>

				<div class="text-md px-4 text-slate-200">Page {page + 1}</div>

				<Button.Root 
					class="p-1 hover:bg-gray-700 disabled:bg-opacity-0 disabled:cursor-not-allowed rounded-lg transition-all group"
					on:click={() => page++}
					disabled={mods.length < pageSize}
				>
					<Icon
					class="text-white text-2xl group-disabled:text-slate-400 align-middle"
						icon="mdi:chevron-right"
					/>
				</Button.Root>
			</div>

			<Popover.Root>
				<Popover.Trigger
					class="flex items-center bg-gray-900 rounded-lg px-3 py-1.5
                                        border border-gray-500 border-opacity-0 hover:border-opacity-100 truncate"
				>
					<Icon class="text-slate-400 mr-2 text-xl flex-shrink-0" icon="mdi:filter" />
					<div class="text-slate-300 text-left mr-2">Filter</div>
					{#if categories.length > 0}
						<div class="text-slate-500 truncate italic">
							{categories.join(', ')}
						</div>
					{/if}
					<Icon class="text-slate-400 ml-2 text-xl flex-shrink-0" icon="mdi:chevron-down" />
				</Popover.Trigger>
				<Popover.Content
					class="flex flex-col bg-gray-800 gap-2 shadow-xl py-4 pl-4 pr-8 rounded-lg border border-gray-600"
					transition={slide}
					transitionConfig={{ duration: 150, easing: quadOut }}
				>
					<ModListSortOption label="Include NSFW" bind:value={includeNsfw} />
					<ModListSortOption label="Include deprecated" bind:value={includeDeprecated} />
					<Separator.Root class="w-full h-[1px] bg-gray-600 my-1" />
					<div class="grid gap-y-2 gap-x-4 grid-cols-3">
						{#each allCategories as category}
							<ModListSortOption
								label={category}
								value={categories.includes(category)}
								onChange={(value) => onFilterChange(category, value)}
							/>
						{/each}
					</div>
				</Popover.Content>
			</Popover.Root>
		</div>

		<div class="flex flex-col flex-grow overflow-y-auto pr-2 pb-3">
			{#if mods.length === 0}
				<div class="text-slate-300 text-lg text-center mt-4">No mods found 😥</div>
			{/if}
			{#each mods.slice(0, pageSize - 1) as mod}
				<ModListItem onClick={onModClicked} {mod} />
			{/each}
		</div>
	</div>

	{#if activeMod}
		<ModDetailsMenu mod={activeMod} onClose={() => (activeMod = undefined)} {extraDropdownOptions}>
			<slot name="details" />
		</ModDetailsMenu>
	{/if}
</div>
