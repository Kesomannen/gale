<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';

	export let value: boolean = false;
	export let onValueChanged: (newValue: boolean) => void = () => {};

	let className = '';

	export { className as class };
</script>

<Checkbox.Root
	bind:checked={value}
	onCheckedChange={(value) => {
		if (value === 'indeterminate') return;
		onValueChanged(value);
	}}
>
	<Checkbox.Indicator
		class="h-6 w-6 rounded-md p-1 {className}
          bg-{value ? 'green-700' : 'gray-800'}
          hover:bg-{value ? 'green-600' : 'gray-700'}
          {!value && 'border border-gray-500'}"
	>
		{#if value}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</Checkbox.Indicator>
</Checkbox.Root>
