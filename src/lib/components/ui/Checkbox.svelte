<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';
	import type { ClassValue } from 'clsx';

	type Props = {
		checked?: boolean;
		disabled?: boolean;
		onCheckedChange?: (newValue: boolean) => void;
		class?: ClassValue;
	};

	let {
		checked = $bindable(false),
		disabled = false,
		onCheckedChange,
		class: classProp
	}: Props = $props();

	let stateClasses = $derived(
		checked
			? [!disabled && 'hover:bg-accent-600', 'bg-accent-700']
			: [!disabled && 'hover:bg-primary-700', 'bg-primary-800 border border-primary-500']
	);
</script>

<Checkbox.Root {disabled} bind:checked {onCheckedChange} class="group">
	<div
		class={[
			classProp,
			stateClasses,
			'size-6 cursor-pointer rounded-md p-1 group-data-[disabled]:cursor-default'
		]}
	>
		{#if checked}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</div>
</Checkbox.Root>
