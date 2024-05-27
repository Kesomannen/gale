<script lang="ts">
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';

	import { SortBy, type Mod, type QueryModsArgs, type DropdownOption, SortOrder } from '$lib/models';

	import Icon from '@iconify/svelte';
	import { Select } from 'bits-ui';

	import { fly, slide } from 'svelte/transition';
	import VirtualList from '@sveltejs/svelte-virtual-list';
	import Checkbox from '$lib/components/Checkbox.svelte';

	const sortOrders = [
		{ value: SortOrder.Descending, label: 'Descending' },
		{ value: SortOrder.Ascending, label: 'Ascending' },
	];

	export let sortOptions: {
		value: SortBy;
		label: string;
	}[];

	let listStart = 0;
	let listEnd = 0;

	let maxCount = 20;
	let searchTerm: string | undefined;
	let categories: string[] = [];
	let includeNsfw = false;
	let includeDeprecated = false;
	let sortBy: { value: SortBy, label: string } = sortOptions[0];
	let sortOrder: { value: SortOrder, label: string } = sortOrders[0];

	export let mods: Mod[] = [];
	export let activeMod: Mod | undefined = undefined;
	export let extraDropdownOptions: DropdownOption[] = [];
	export let queryArgs: QueryModsArgs;

	$: queryArgs = {
		maxCount,
		searchTerm,
		categories,
		includeNsfw,
		includeDeprecated,
		includeDisabled: true,
		sortOrder: sortOrder.value,
		sortBy: sortBy.value
	};

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

	$: if (listEnd > mods.length - 1 && mods.length === maxCount) {
		maxCount += 20;
	}

	$: {
		mods;
		console.log("mods changed");
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex flex-col flex-grow w-[60%] pt-3 pl-3 overflow-hidden">
		<div class="flex gap-1 mb-2 pr-3">
			<div class="relative flex-grow">
				<input
					type="text"
					class="w-full py-2 pr-10 pl-12 rounded-lg bg-gray-900 text-slate-300 truncate"
					bind:value={searchTerm}
					placeholder="Search for mods..."
				/>
				<Icon class="absolute left-[12px] top-[9px] text-slate-400 text-2xl" icon="mdi:magnify" />
			</div>

			<Select.Root items={sortOrders} bind:selected={sortOrder}>
				<Select.Trigger
					class="flex items-center flex-shrink-0 w-44 bg-gray-900 rounded-lg px-3 
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-2xl mr-2" icon="mdi:sort" />
					<Select.Value class="text-slate-300 text-left w-full" />
					<Icon class="text-slate-400 text-2xl ml-auto" icon="mdi:chevron-down" />
				</Select.Trigger>
				<Select.Content
					class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-48 rounded-lg border border-gray-600"
					transition={fly}
					transitionConfig={{ duration: 50 }}
				>
					{#each sortOrders as item}
						<Select.Item
							value={item.value}
							class="flex items-center px-3 py-1 text-slate-400 hover:text-slate-200 text-left rounded-lg hover:bg-gray-700 cursor-default"
						>
							{item.label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>

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
					transition={fly}
					transitionConfig={{ duration: 50 }}
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

		<div class="flex gap-2 pr-4 pl-1 pb-2 text-slate-300">
			<Checkbox bind:value={includeNsfw} /> <span class="mr-3">Show NSFW</span>
			<Checkbox bind:value={includeDeprecated} /> <span>Show deprecated</span>
		</div>

		<slot name="header" />

		{#if mods.length === 0}
			<div class="text-slate-300 text-lg text-center mt-4">No mods found 😥</div>
		{:else}
			<VirtualList
				itemHeight={48 + 16}
				items={mods}
				let:item
				bind:start={listStart}
				bind:end={listEnd}
			>
				<ModListItem onClick={onModClicked} mod={item} isSelected={activeMod == item}>
					<slot name="item" mod={item} />
				</ModListItem>
			</VirtualList>
		{/if}
	</div>

	{#if activeMod}
		<ModDetailsMenu mod={activeMod} onClose={() => (activeMod = undefined)} {extraDropdownOptions}>
			<slot name="details" />
		</ModDetailsMenu>
	{/if}
</div>
