<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';
	import type { ClassValue } from 'clsx';

	type Props = {
		id?: string;
		checked?: boolean;
		indeterminate?: boolean;
		disabled?: boolean;
		onCheckedChange?: (newValue: boolean) => void;
		class?: ClassValue;
	};

	let {
		id,
		checked = $bindable(false),
		indeterminate = false,
		disabled = false,
		onCheckedChange,
		class: classProp
	}: Props = $props();

	let stateClasses = $derived(
		checked || indeterminate
			? [
					!disabled && 'hover:bg-accent-500 dark:hover:bg-accent-600',
					'bg-accent-600 dark:bg-accent-700'
				]
			: [
					!disabled && 'hover:bg-primary-300 dark:hover:bg-primary-700 hover:border-primary-400',
					'bg-primary-100 border border-primary-200 dark:bg-primary-800 dark:border-primary-500'
				]
	);
</script>

<Checkbox.Root {id} {disabled} bind:checked {indeterminate} {onCheckedChange} class="group">
	<div
		class={[
			classProp,
			stateClasses,
			'size-6.5 cursor-pointer rounded-md p-1 group-data-disabled:cursor-default'
		]}
	>
		{#if indeterminate}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:minus" />
		{:else if checked}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</div>
</Checkbox.Root>
