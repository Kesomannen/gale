<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';
	import type { ClassValue } from 'clsx';

	type Props = {
		id?: string;
		checked?: boolean;
		disabled?: boolean;
		size?: 'sm' | 'md';
		onCheckedChange?: (newValue: boolean) => void;
		class?: ClassValue;
	};

	let {
		id,
		checked = $bindable(false),
		disabled = false,
		size = 'md',
		onCheckedChange,
		class: classProp
	}: Props = $props();

	let stateClasses = $derived(
		checked
			? [
					!disabled && 'hover:bg-accent-500 dark:hover:bg-accent-600',
					'bg-accent-600 dark:bg-accent-700'
				]
			: [
					!disabled && 'hover:bg-primary-300 dark:hover:bg-primary-700 hover:border-primary-400',
					'bg-primary-100 border border-primary-200 dark:bg-primary-800 dark:border-primary-500'
				]
	);
	let sizeClasses = $derived(size === 'sm' ? 'size-5 rounded-sm p-0.5' : 'size-6.5 rounded-md p-1');
</script>

<Checkbox.Root {id} {disabled} bind:checked {onCheckedChange} class="group">
	<div
		class={[
			classProp,
			stateClasses,
			sizeClasses,
			'cursor-pointer group-data-disabled:cursor-default'
		]}
	>
		{#if checked}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</div>
</Checkbox.Root>
