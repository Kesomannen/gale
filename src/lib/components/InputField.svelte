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
	class="{className} valid:focus:ring-accent-500 disabled:cursor-not-alloweddisabled:text-slate-400 min-w-0 grow rounded-lg border border-transparent bg-slate-900 px-3 py-1 text-slate-300 placeholder-slate-400 invalid:ring-2 invalid:ring-red-500 focus:border-transparent focus:ring-2 focus:outline-hidden enabled:hover:border-slate-500 text-{size} placeholder:text-{size}"
/>
