<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { ClassValue } from 'clsx';
	import type { MouseEventHandler } from 'svelte/elements';
	import Tooltip from './Tooltip.svelte';

	type Props = {
		icon: string;
		label: string;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		class?: ClassValue;
		color?: 'primary' | 'accent' | 'red';
		showTooltip?: boolean;
	};

	let {
		icon,
		label,
		onclick,
		class: classProp,
		color = 'primary',
		showTooltip = false
	}: Props = $props();

	let colorClasses = $derived(
		{
			primary:
				'text-primary-500 hover:bg-primary-200 hover:text-primary-700 dark:text-primary-400 dark:hover:bg-primary-600 dark:hover:text-primary-300',
			accent:
				'text-primary-500 hover:bg-accent-100 hover:text-accent-700 dark:text-primary-400 dark:hover:bg-accent-700 dark:hover:text-accent-300',
			red: 'text-primary-500 hover:bg-red-100 hover:text-red-600 dark:text-primary-400 dark:hover:bg-red-800 dark:hover:text-red-300'
		}[color]
	);
</script>

{#snippet button()}
	<button
		class={[classProp, colorClasses, 'shrink-0 rounded-sm p-1.5']}
		aria-label={label}
		{onclick}
	>
		<Icon {icon} />
	</button>
{/snippet}

{#if showTooltip}
	<Tooltip text={label}>
		{@render button()}
	</Tooltip>
{:else}
	{@render button()}
{/if}
