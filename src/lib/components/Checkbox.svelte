<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';

	type Props = {
		value?: boolean;
		disabled?: boolean;
		onValueChanged?: (newValue: boolean) => void;
		class?: string;
	};

	let {
		value = $bindable(false),
		disabled = false,
		onValueChanged = () => {},
		class: className = ''
	}: Props = $props();

	let stateClasses = $derived(
		value
			? 'bg-accent-700 hover:bg-accent-600'
			: 'bg-primary-800 hover:bg-primary-700 border border-primary-500'
	);
</script>

<Checkbox.Root
	{disabled}
	bind:checked={value}
	onCheckedChange={(value) => {
		if (value === 'indeterminate') return;
		onValueChanged(value);
	}}
>
	<Checkbox.Indicator
		class="{stateClasses} {className} size-6 rounded-md p-1 disabled:cursor-not-allowed"
	>
		{#if value}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</Checkbox.Indicator>
</Checkbox.Root>
