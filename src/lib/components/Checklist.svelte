<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';

	let className = '';

	export let title: string;
	export let items: T[];

	export let getLabel: (item: T, index: number) => string;
	export let get: (index: number) => boolean;
	export let set: (index: number, value: boolean) => void;

	export { className as class };
</script>

<div class="rounded-lg overflow-hidden border border-gray-900 {className}">
	<div
		class="flex items-center gap-3 px-3 py-2 text-slate-300 bg-gray-950 font-bold"
	>
		<Checkbox
			value={items.every((_, i) => get(i))}
			onValueChanged={(newValue) => items.forEach((_, i) => set(i, newValue))}
		/>
		{title}
	</div>

	{#each items as value, i}
		<div
			class="flex items-center gap-3 px-3 py-1.5 text-slate-300 odd:bg-gray-900"
		>
			<Checkbox value={get(i)} onValueChanged={(newValue) => set(i, newValue)} />

			{getLabel(value, i)}
		</div>
	{/each}
</div>
