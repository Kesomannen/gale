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
	import { Button, DropdownMenu, Select } from 'bits-ui';

	import VirtualList from '@sveltejs/svelte-virtual-list';
	import { categories } from '$lib/profile';
	import { Root } from 'postcss';
	import { scale, slide } from 'svelte/transition';

	export let sortOptions: SortBy[];

	let listStart = 0;
	let listEnd = 0;

	let maxCount = 20;
	let searchTerm: string | undefined;
	let includeCategories: string[] = [];
	let excludeCategories: string[] = [];
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
		includeCategories,
		excludeCategories,
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

	$: if (listEnd > mods.length - 2 && mods.length === maxCount) {
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

			<Dropdown items={sortOptions} bind:selected={sortBy}>
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
				items={categories.filter((category) => !excludeCategories.includes(category)).toSorted()}
				getLabel={(category) => category}
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
						<span class="text-slate-300">Include categories</span>
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
				items={categories.filter((category) => !includeCategories.includes(category)).toSorted()}
				getLabel={(category) => category}
				multiple={true}
				bind:selected={excludeCategories}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex items-center w-1/2 bg-gray-900 rounded-lg px-3 py-1.5 overflow-hidden
							border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-lg mr-2 flex-shrink-0" icon="mdi:filter" />
					{#if excludeCategories.length === 0}
						<span class="text-slate-300">Exclude categories</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each excludeCategories as category}
								<div class="bg-red-600 text-white rounded-full pl-3 pr-0.5 py-0.5 text-sm">
									<span class="truncate overflow-hidden">{category}</span>

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
