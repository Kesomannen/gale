<script lang="ts">
	import type { Mod, QueryModsArgsWithoutMax } from '$lib/types';
	import type { Snippet } from 'svelte';
	import games from '$lib/state/game.svelte';

	const itemHeight = 66;

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

	let listEnd = $state(0);
	let list: HTMLDivElement | null = $state(null);

	$effect(() => {
		if (listEnd > mods.length - 4 && mods.length === maxCount) {
			maxCount += 20;
		}
	});

	$effect(() => {
		queryArgs;
		list?.scrollTo(0, 0);
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

	function onscroll() {
		if (!list) return;

		let scrollTop = list.scrollTop;
		let visibleCount = Math.ceil(list.clientHeight / itemHeight);

		listEnd = Math.min(mods.length, Math.floor(scrollTop / itemHeight) + visibleCount);
	}
</script>

{#if mods.length === 0}
	<div class="text-primary-300 text-center">
		{@render placeholder?.()}
	</div>
{:else}
	<div class="overflow-y-auto" bind:this={list} {onscroll}>
		{#each mods as mod, index (mod.uuid)}
			{@render item({
				mod,
				index,
				isSelected: selected?.uuid === mod.uuid
			})}
		{/each}
	</div>
{/if}
