<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';

	export let value: boolean = false;
	export let onValueChanged: (newValue: boolean) => void = () => {};

	let className = '';

	export { className as class };

	$: stateClasses = value
		? 'bg-accent-700 hover:bg-accent-600'
		: 'bg-primary-800 hover:bg-primary-700 border border-primary-500';
</script>

<Checkbox.Root
	bind:checked={value}
	onCheckedChange={(value) => {
		if (value === 'indeterminate') return;
		onValueChanged(value);
	}}
>
	<Checkbox.Indicator class="{stateClasses} {className} size-6 rounded-md p-1">
		{#if value}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</Checkbox.Indicator>
</Checkbox.Root>
