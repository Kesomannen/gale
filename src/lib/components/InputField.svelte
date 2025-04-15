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
	class="{className} valid:focus:ring-accent-500 disabled:text-primary-400 bg-primary-900 text-primary-300 placeholder-primary-400 hover:ring-primary-500 min-w-0 grow rounded-lg px-3 py-1 invalid:ring-2 invalid:ring-red-500 hover:ring-1 focus:ring-2 focus:outline-hidden disabled:cursor-not-allowed text-{size} placeholder:text-{size}"
/>
