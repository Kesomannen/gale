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
		// If `set` is expensive, setAll can be provided to batch the set operation.
		// If not provided, setAll will be implemented by calling `set` for each item.
		setAll?: (value: boolean) => void;
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
		setAll,
		getLabel = (item, _) => item as unknown as string,
		item: itemSnippet
	}: Props = $props();

	function toggleAll() {
		const allChecked = items.every((item, i) => get(item, i));
		if (setAll) {
			setAll(!allChecked);
		} else {
			items.forEach((item, i) => set(item, i, !allChecked));
		}
	}
</script>

<div
	class={[
		classProp,
		'border-primary-200 dark:border-primary-900 relative flex flex-col overflow-hidden rounded-lg border'
	]}
>
	<label
		class="text-primary-900 dark:bg-primary-900 bg-primary-100 flex w-full shrink-0 items-center px-4 py-2.5 font-medium dark:text-white"
	>
		<Checkbox
			class="mr-3"
			checked={items.every((item, i) => get(item, i))}
			onCheckedChange={toggleAll}
		/>
		{title}
	</label>

	<div class="overflow-auto" class:max-h-96={maxHeight === 'sm'}>
		{#each items as item, i}
			<label
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
			</label>
		{/each}
	</div>
</div>
