<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Label from './Label.svelte';

	export let label: string | undefined = undefined;
	export let placeholder: string = '';
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
		{placeholder}
		on:keydown={(e) => {
			if (e.key === 'Enter') {
				dispatch('submit', value);
			}
		}}
		class="ml-auto flex-grow px-3 py-1 rounded-lg bg-gray-900 placeholder-slate-400
			 text-slate-300 hover:text-slate-200 border border-slate-500 border-opacity-0 hover:border-opacity-100"
	/>
</div>
