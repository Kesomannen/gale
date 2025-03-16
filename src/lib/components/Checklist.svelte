<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';

	let className = '';

	export let title: string;
	export let items: T[];
	export let maxHeight: 'none' | 'sm' = 'none';

	export let get: (item: T, index: number) => boolean;
	export let set: (item: T, index: number, value: boolean) => void;
	export let getLabel: (item: T, index: number) => string = (item, _) => item as unknown as string;

	export { className as class };
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

				<slot {item} index={i}>
					{getLabel(item, i)}
				</slot>
			</div>
		{/each}
	</div>
</div>
