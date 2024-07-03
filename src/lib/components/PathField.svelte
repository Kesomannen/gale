<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import Label from './Label.svelte';

	export let label: string = '';
	export let onClick: () => void;
	export let value: string | null;
	export let icon: string = 'mdi:folder';

	$: hasValue = value && value.length > 0;
</script>

<div class="flex items-center">
	{#if label}
		<Label text={label}>
			<slot />
		</Label>
	{/if}

	<Button.Root
		class="flex flex-grow px-3 py-1 items-center text-right rounded-lg group bg-gray-900 truncate
            border border-gray-500 border-opacity-0 hover:border-opacity-100"
		on:click={onClick}
	>
		<div class="mr-2 rounded">
			<Icon {icon} class="align-middle text-slate-300 group-hover:text-slate-200" />
		</div>

		<div
			class="text-slate-300 group-hover:text-slate-200 truncate"
			style="direction: rtl;"
		>
			{hasValue ? value : 'Not set'}
		</div>

		<slot name="field" />
	</Button.Root>
</div>
