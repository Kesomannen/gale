<script lang="ts">
	import { Label } from 'bits-ui';

	export let label: string | undefined = undefined;
	export let placeholder: string = '';
	export let value: string = '';
	export let size: 'sm' | 'md' | 'lg' = 'md';
</script>

<div class="flex items-center cursor-auto flex-grow text-{size}">
	{#if label}
		<Label.Root class="text-slate-300 w-36">{label}</Label.Root>
	{/if}

	<input
		type="text"
		bind:value
		{placeholder}
		class="ml-auto flex-grow px-3 py-1 rounded-lg bg-gray-900
			 text-slate-300 hover:text-slate-100 border border-gray-500 border-opacity-0 hover:border-opacity-100"
	/>
</div>
