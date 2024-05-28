<script lang="ts">
	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';

	import {
		SortBy,
		type Mod,
		type QueryModsArgs,
		type DropdownOption,
		SortOrder
	} from '$lib/models';

	import Icon from '@iconify/svelte';
	import { Select } from 'bits-ui';

	import VirtualList from '@sveltejs/svelte-virtual-list';

	export let sortOptions: SortBy[];

	let listStart = 0;
	let listEnd = 0;

	let maxCount = 20;
	let searchTerm: string | undefined;
	let categories: string[] = [];
	let includeNsfw = false;
	let includeDeprecated = false;
	let sortBy = sortOptions[0];
	let sortOrder = SortOrder.Descending;

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
		sortOrder,
		sortBy
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
		console.log('mods changed');
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex flex-col flex-grow w-[60%] pt-3 pl-3 overflow-hidden">
		<div class="flex gap-1.5 mb-1.5 pr-3">
			<div class="relative flex-grow">
				<input
					type="text"
					class="w-full py-1.5 px-11 rounded-lg bg-gray-900 text-slate-300 placeholder-slate-400 truncate
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
					bind:value={searchTerm}
					placeholder="Search for mods..."
				/>
				<Icon class="absolute left-[12px] top-[8px] text-slate-500 text-2xl" icon="mdi:magnify" />
			</div>

			<Dropdown items={[SortOrder.Descending, SortOrder.Ascending]} bind:selected={sortOrder}>
				<Select.Trigger
					let:text
					let:open
					slot="trigger"
					class="flex items-center flex-shrink-0 w-48 bg-gray-900 rounded-lg px-3 
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon
						class="text-slate-400 text-2xl mr-2"
						icon={sortOrder === SortOrder.Descending ? 'mdi:sort-descending' : 'mdi:sort-ascending'}
					/>
					<div class="text-slate-300 text-left w-full">{text}</div>
					<Icon
						class="text-slate-400 text-xl transition-all flex-shrink-0 duration-100 ease-out
					transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>

			<Dropdown items={sortOptions} bind:selected={sortBy}>
				<Select.Trigger
					let:text
					let:open
					slot="trigger"
					class="flex items-center flex-shrink-0 w-48 bg-gray-900 rounded-lg px-3 
								border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-2xl mr-2" icon="mdi:sort" />
					<div class="text-slate-300 text-left w-full">{text}</div>
					<Icon
						class="text-slate-400 text-xl transition-all flex-shrink-0 duration-100 ease-out
					transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
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
