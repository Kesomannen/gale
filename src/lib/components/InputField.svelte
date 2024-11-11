<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let value: string = '';
	export let size: 'sm' | 'md' | 'lg' = 'md';

	let className = '';

	export { className as class };

	const dispatch = createEventDispatcher<{
		submit: string;
		change: string;
	}>();
</script>

<input
	type="text"
	bind:value
	autocomplete="off"
	{...$$restProps}
	on:keydown={(evt) => {
		if (evt.key === 'Enter') {
			dispatch('submit', value);
		}
	}}
	on:change={() => dispatch('change', value)}
	class="{className} min-w-0 flex-grow rounded-lg border border-slate-500 border-opacity-0 bg-slate-900 px-3 py-1 text-slate-300 placeholder-slate-400 invalid:ring-2 invalid:ring-red-500 focus:border-opacity-0 focus:outline-none focus:ring-2 valid:focus:ring-accent-500 enabled:hover:border-opacity-100 disabled:cursor-not-allowed disabled:text-slate-400 text-{size} placeholder:text-{size}"
/>
