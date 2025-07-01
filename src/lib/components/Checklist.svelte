<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		class?: string;
		title: string;
		items: T[];
		maxHeight?: 'none' | 'sm';
		get: (item: T, index: number) => boolean;
		set: (item: T, index: number, value: boolean) => void;
		getLabel?: (item: T, index: number) => string;
		item?: Snippet<[{ item: T; index: number }]>;
	};

	let {
		class: classProp = '',
		title,
		items,
		maxHeight = 'none',
		get,
		set,
		getLabel = (item, _) => item as unknown as string,
		item: itemSnippet
	}: Props = $props();
</script>

<div class={[classProp, 'border-primary-900 relative overflow-hidden rounded-lg border-2']}>
	<div class="bg-primary-950 text-primary-300 flex w-full items-center px-3 py-2 font-bold">
		<Checkbox
			class="mr-3"
			checked={items.every((item, i) => get(item, i))}
			onCheckedChange={(newValue) => items.forEach((item, i) => set(item, i, newValue))}
		/>
		{title}
	</div>

	<div
		class="overflow-x-hidden"
		class:overflow-y-auto={maxHeight !== 'none'}
		class:max-h-96={maxHeight === 'sm'}
	>
		{#each items as item, i}
			<div class="text-primary-300 even:bg-primary-900 flex items-center px-3 py-1.5">
				<Checkbox
					class="mr-3"
					checked={get(item, i)}
					onCheckedChange={(newValue) => set(item, i, newValue)}
				/>

				{#if itemSnippet}{@render itemSnippet({ item, index: i })}{:else}
					{getLabel(item, i)}
				{/if}
			</div>
		{/each}
	</div>
</div>
