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
			primary: 'text-primary-400 hover:bg-primary-600 hover:text-primary-300',
			accent: 'text-primary-400 hover:bg-accent-700 hover:text-accent-300',
			red: 'text-primary-400 hover:bg-red-800 hover:text-red-300'
		}[color]
	);
</script>

{#snippet button()}
	<button class={[classProp, colorClasses, 'shrink-0 rounded-sm p-1']} aria-label={label} {onclick}>
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
