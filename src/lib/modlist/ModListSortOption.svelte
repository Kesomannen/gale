<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox, Label } from 'bits-ui';

	export let label: string;
	export let value: boolean = false;
	export let onChange: (value: boolean) => void = () => {};

	function onCheckedChange(value: boolean | 'indeterminate') {
		onChange(value as boolean);
	}
</script>

<Checkbox.Root 
	id="checkbox"
	class="inline-flex gap-2"
	bind:checked={value} 
	{onCheckedChange}
>
	<Checkbox.Indicator
		class="rounded-md w-5 h-5 p-1 
          bg-{value ? 'green-600' : 'gray-800'}
          hover:bg-{value ? 'green-500' : 'gray-700'}
          {value ? '' : 'border border-gray-500'}"
	>
		{#if value}
			<Icon class="text-slate-200 w-full h-full" icon="mdi:check" />
		{/if}
	</Checkbox.Indicator>
	<Label.Root for="checkbox">
		<Checkbox.Input />
		<div class="text-slate-400 text-sm">{label}</div>
	</Label.Root>
</Checkbox.Root>
