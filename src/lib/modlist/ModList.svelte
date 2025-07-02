<script lang="ts">
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { type SortBy, type Mod, type QueryModsArgsWithoutMax } from '$lib/types';
	import type { Writable } from 'svelte/store';
	import { activeGame } from '$lib/stores.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		mods: Mod[];
		maxCount: number;
		queryArgs: Writable<QueryModsArgsWithoutMax>;
		selected: Mod | null;
		placeholder?: Snippet;
		item: Snippet<[{ mod: Mod; index: number; isSelected: boolean }]>;
	};

	let {
		mods,
		maxCount = $bindable(20),
		queryArgs,
		selected = $bindable(),
		placeholder,
		item
	}: Props = $props();

	let listStart = $state(0);
	let listEnd = $state(0);
	let virtualList: VirtualList<Mod> | null = $state(null);

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
</script>

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
			{@render item({
				mod,
				index,
				isSelected: selected?.uuid === mod.uuid
			})}
		{/snippet}
	</VirtualList>
{/if}
