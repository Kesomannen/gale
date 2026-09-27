<script lang="ts" generics="T">
	import VirtualList from '$lib/components/ui/VirtualList.svelte';
	import type { Mod, QueryModsArgsWithoutMax } from '$lib/types';
	import type { Snippet } from 'svelte';

	type Props = {
		mods: T[];
		maxCount: number;
		queryArgs: QueryModsArgsWithoutMax;
		placeholder?: Snippet;
		item: Snippet<[{ mod: T; index: number }]>;
		rowId?: (mod: T) => string;
	};

	let { mods, maxCount = $bindable(20), queryArgs, placeholder, item, rowId }: Props = $props();

	let listStart = $state(0);
	let listEnd = $state(0);
	let virtualList: VirtualList<T, string> | null = $state(null);

	$effect(() => {
		if (listEnd > mods.length - 4 && mods.length === maxCount) {
			maxCount += 20;
		}
	});

	$effect(() => {
		JSON.stringify(queryArgs);
		virtualList?.scrollTo(0);
	});
</script>

{#if mods.length === 0}
	<div class="text-primary-600 dark:text-primary-300 text-center">
		{@render placeholder?.()}
	</div>
{:else}
	<VirtualList
		{rowId}
		items={mods}
		bind:this={virtualList}
		bind:start={listStart}
		bind:end={listEnd}
	>
		{#snippet children({ item: mod, index })}
			{@render item({
				mod,
				index
			})}
		{/snippet}
	</VirtualList>
{/if}
