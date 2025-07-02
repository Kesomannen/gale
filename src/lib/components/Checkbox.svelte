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
			? 'bg-accent-700 hover:bg-accent-600'
			: 'bg-primary-800 hover:bg-primary-700 border border-primary-500'
	);
</script>

<Checkbox.Root {disabled} bind:checked {onCheckedChange}>
	<div
		class={[
			classProp,
			stateClasses,
			'size-6 cursor-pointer rounded-md p-1 disabled:cursor-not-allowed'
		]}
	>
		{#if checked}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</div>
</Checkbox.Root>
