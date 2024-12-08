<script lang="ts">
	import ModDetails from '$lib/modlist/ModDetails.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { communityUrl, sentenceCase } from '$lib/util';
	import {
		SortBy,
		type Mod,
		type QueryModsArgs,
		SortOrder,
		type ModContextItem
	} from '$lib/models';
	import type { Writable } from 'svelte/store';
	import ModListCategoryFilter from './ModListCategoryFilter.svelte';
	import { activeGame } from '$lib/stores';

	export let mods: Mod[] = [];
	export let maxCount = 20;

	export let selected: Mod | null;

	$: allContextItems = [...contextItems, ...defaultContextItems];

	let listStart = 0;
	let listEnd = 0;
	let virtualList: VirtualList<Mod>;

	$: if (listEnd > mods.length - 2 && mods.length === maxCount) {
		maxCount += 20;
		console.log('increasing max count');
	}

	$: {
		$queryArgs;
		virtualList?.scrollTo(0);
	}

	$: {
		$activeGame;
		selected = null;
	}

	export function selectMod(mod: Mod) {
		if (selected === null || selected.uuid !== mod.uuid) {
			selected = mod;
		} else {
			selected = null;
		}
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div class="flex w-[60%] flex-grow flex-col overflow-hidden pl-3 pt-3">
		{#if mods.length === 0}
			<div class="mt-4 text-center text-lg text-slate-300">No mods found 😥</div>
		{:else}
			<VirtualList
				itemHeight={66}
				items={mods}
				bind:this={virtualList}
				bind:start={listStart}
				bind:end={listEnd}
				let:item={mod}
				let:index
			>
				<slot
					name="item"
					data={{
						mod,
						index,
						contextItems: allContextItems,
						isSelected: selected?.uuid === mod.uuid
					}}
				/>
			</VirtualList>
		{/if}
	</div>

	{#if selected !== null}
		<ModDetails mod={selected} contextItems={allContextItems} on:close={() => (selected = null)}>
			<slot name="details" />
		</ModDetails>
	{/if}
</div>
