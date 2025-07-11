<script lang="ts">
	import VirtualList from '$lib/components/ui/VirtualList.svelte';
	import type { Mod, QueryModsArgsWithoutMax } from '$lib/types';
	import type { Writable } from 'svelte/store';
	import type { Snippet } from 'svelte';
	import games from '$lib/state/game.svelte';

	type Props = {
		mods: Mod[];
		maxCount: number;
		queryArgs: QueryModsArgsWithoutMax;
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
	let virtualList: VirtualList<Mod, string> | null = $state(null);

	$effect(() => {
		if (listEnd > mods.length - 2 && mods.length === maxCount) {
			maxCount += 20;
		}
	});

	$effect(() => {
		queryArgs;
		virtualList?.scrollTo(0);
	});

	$effect(() => {
		games.active;
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
	<div class="text-primary-300 text-center">
		{@render placeholder?.()}
	</div>
{:else}
	<VirtualList
		itemHeight={66}
		items={mods}
		rowId={(mod) => mod.uuid}
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
