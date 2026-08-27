<script lang="ts" generics="T">
	import type { ClassValue } from 'clsx';
	import Checkbox from './Checkbox.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		class?: ClassValue;
		title: string;
		items: T[];
		maxHeight?: 'none' | 'sm';
		get: (item: T, index: number) => boolean;
		set: (item: T, index: number, value: boolean) => void;
		getLabel?: (item: T, index: number) => string;
		item?: Snippet<[{ item: T; index: number }]>;
	};

	let {
		class: classProp,
		title,
		items,
		maxHeight = 'none',
		get,
		set,
		getLabel = (item, _) => item as unknown as string,
		item: itemSnippet
	}: Props = $props();
</script>

<div
	class={[
		classProp,
		'border-primary-200 dark:border-primary-900 relative overflow-hidden rounded-lg border'
	]}
>
	<label
		class="text-primary-900 dark:bg-primary-900 bg-primary-100 flex w-full items-center px-4 py-2.5 font-medium dark:text-white"
	>
		<Checkbox
			class="mr-3"
			checked={items.every((item, i) => get(item, i))}
			onCheckedChange={(newValue) => items.forEach((item, i) => set(item, i, newValue))}
		/>
		{title}
	</label>

	<div
		class="overflow-x-hidden"
		class:overflow-y-auto={maxHeight !== 'none'}
		class:max-h-96={maxHeight === 'sm'}
	>
		{#each items as item, i}
			<div
				class="text-primary-700 dark:text-primary-300 dark:even:bg-primary-900/30 even:bg-primary-100 flex items-center px-4 py-2"
			>
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
