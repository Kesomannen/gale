<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Label from './Label.svelte';

	export let label: string | undefined = undefined;
	export let value: string = '';
	export let size: 'sm' | 'md' | 'lg' = 'md';

	const dispatch = createEventDispatcher<{
		submit: string;
	}>();
</script>

<div class="flex items-center cursor-auto text-{size}">
	{#if label}
		<Label text={label}>
			<slot />
		</Label>
	{/if}

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
		class="ml-auto flex-grow px-3 py-1 rounded-lg bg-gray-900 placeholder-slate-400
			 text-slate-300 hover:text-slate-200
			 valid:focus:ring-green-400 invalid:ring-red-500 focus:ring-2 invalid:ring-2 focus:outline-none
			  border border-slate-500 border-opacity-0 valid:hover:border-opacity-100 focus:border-opacity-0"
	/>
</div>
