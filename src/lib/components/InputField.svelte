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
	on:keydown={(e) => {
		if (e.key === 'Enter') {
			dispatch('submit', value);
		}
	}}
	on:change={() => {
		dispatch('change', value);
	}}
	class="ml-auto w-full flex-grow px-3 py-1 rounded-lg bg-gray-900 placeholder-slate-400 text-slate-300
		 valid:focus:ring-green-400 invalid:ring-red-500 focus:ring-2 invalid:ring-2 focus:outline-none
		  border border-slate-500 border-opacity-0 valid:hover:border-opacity-100 focus:border-opacity-0
		  disabled:text-slate-400 text-{size} placeholder:text-{size} {className}"
/>
