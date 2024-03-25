<script lang="ts">
	import { Label } from 'bits-ui';
	
	export let label: string | undefined = undefined;
	export let pattern: string | undefined = undefined;
	export let placeholder: string = '';
	export let value: string = '';
</script>

<div class="flex items-center cursor-auto">
	{#if label}
		<Label.Root class="text-slate-300 text-md">{label}</Label.Root>
	{/if}

	<input
		type="text"
		bind:value
		{placeholder}
		{pattern}
		class="ml-auto w-{label ? '80' : 'full'} px-3 py-2 rounded-md bg-gray-900 text-sm
					text-slate-300 hover:text-slate-100 border border-gray-500 border-opacity-0 hover:border-opacity-100"
	/>
</div>
