<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';

	type Props = {
		class?: string;
		title: string;
		items: T[];
		maxHeight?: 'none' | 'sm';
		get: (item: T, index: number) => boolean;
		set: (item: T, index: number, value: boolean) => void;
		getLabel?: (item: T, index: number) => string;
		children?: import('svelte').Snippet<[any]>;
	};

	let {
		class: className = '',
		title,
		items,
		maxHeight = 'none',
		get,
		set,
		getLabel = (item, _) => item as unknown as string,
		children
	}: Props = $props();
</script>

<div class="border-primary-900 relative overflow-hidden rounded-lg border-2 {className}">
	<div class="bg-primary-950 text-primary-300 flex w-full items-center px-3 py-2 font-bold">
		<Checkbox
			class="mr-3"
			value={items.every((item, i) => get(item, i))}
			onValueChanged={(newValue) => items.forEach((item, i) => set(item, i, newValue))}
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
					value={get(item, i)}
					onValueChanged={(newValue) => set(item, i, newValue)}
				/>

				{#if children}{@render children({ item, index: i })}{:else}
					{getLabel(item, i)}
				{/if}
			</div>
		{/each}
	</div>
</div>
