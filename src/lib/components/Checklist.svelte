<script lang="ts" generics="T">
	import Checkbox from './Checkbox.svelte';

	let className = '';

	export let title: string;
	export let items: T[];

	export let getLabel: (item: T, index: number) => string;
	export let get: (item: T, index: number) => boolean;
	export let set: (item: T, index: number, value: boolean) => void;

	export { className as class };
</script>

<div class="rounded-lg overflow-hidden border border-gray-900 {className}">
	<div
		class="flex items-center gap-3 px-3 py-2 text-slate-300 bg-gray-950 font-bold"
	>
		<Checkbox
			value={items.every((item, i) => get(item, i))}
			onValueChanged={(newValue) => items.forEach((item, i) => set(item, i, newValue))}
		/>
		{title}
	</div>

	{#each items as item, i}
		<div
			class="flex items-center gap-3 px-3 py-1.5 text-slate-300 odd:bg-gray-900"
		>
			<Checkbox value={get(item, i)} onValueChanged={(newValue) => set(item, i, newValue)} />

			{getLabel(item, i)}
		</div>
	{/each}
</div>
