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
	class="{className} valid:focus:ring-accent-500 disabled:cursor-not-alloweddisabled:text-primary-400 bg-primary-900 text-primary-300 placeholder-primary-400 enabled:hover:border-primary-500 min-w-0 grow rounded-lg border border-transparent px-3 py-1 invalid:ring-2 invalid:ring-red-500 focus:border-transparent focus:ring-2 focus:outline-hidden text-{size} placeholder:text-{size}"
/>
