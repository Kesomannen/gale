<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';

	let className = '';

	export let title: string;
	export let items: T[];

	export let get: (item: T, index: number) => boolean;
	export let set: (item: T, index: number, value: boolean) => void;
	export let getLabel: (item: T, index: number) => string = (item, _) => item as unknown as string;

	export { className as class };
</script>

<div class="overflow-hidden rounded-lg border border-gray-900 {className}">
	<div class="flex items-center bg-gray-950 px-3 py-2 font-bold text-slate-300">
		<Checkbox
			class="mr-3"
			value={items.every((item, i) => get(item, i))}
			onValueChanged={(newValue) => items.forEach((item, i) => set(item, i, newValue))}
		/>
		{title}
	</div>

	{#each items as item, i}
		<div class="flex items-center px-3 py-1.5 text-slate-300 odd:bg-gray-900">
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
