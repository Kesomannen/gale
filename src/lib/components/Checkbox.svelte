<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';

	export let value: boolean = false;
	export let onValueChanged: (newValue: boolean) => void = () => {};
</script>

<Checkbox.Root
	bind:checked={value}
	onCheckedChange={(value) => {
		if (value === 'indeterminate') return;
		onValueChanged(value);
	}}
>
	<Checkbox.Indicator
		class="rounded-md w-6 h-6 p-1 
          bg-{value ? 'green-700' : 'gray-800'}
          hover:bg-{value ? 'green-600' : 'gray-700'}
          {value ? '' : 'border border-gray-500'}"
	>
		{#if value}
			<Icon class="text-white w-full h-full font-bold" icon="mdi:check" />
		{/if}
	</Checkbox.Indicator>
</Checkbox.Root>
