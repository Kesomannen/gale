<script lang="ts">
	import { dropTransitionTo } from '$lib/transitions';
	import { Tooltip } from 'bits-ui';

	type Props = {
		text?: string;
		side?: 'top' | 'right' | 'bottom' | 'left';
		sideOffset?: number;
		openDelay?: number;
		disabled?: boolean;
		class?: string;
		children?: import('svelte').Snippet;
		tooltip?: import('svelte').Snippet;
	};

	let {
		text = '[No text provided]',
		side = 'top',
		sideOffset = 0,
		openDelay = 150,
		disabled = false,
		class: triggerClass = '',
		children,
		tooltip
	}: Props = $props();

	const distances = {
		top: { x: 0, y: 7 },
		right: { x: -7, y: 0 },
		bottom: { x: 0, y: -7 },
		left: { x: 7, y: 0 }
	};

	let distance = $derived(distances[side]);
</script>

<Tooltip.Root {openDelay}>
	<Tooltip.Trigger class={triggerClass} {disabled}>
		{@render children?.()}
	</Tooltip.Trigger>
	<Tooltip.Content
		class="border-primary-600 bg-primary-800 text-primary-300 max-w-lg cursor-help rounded-lg border px-4 py-2 shadow-lg"
		{...dropTransitionTo(distance)}
		{sideOffset}
		{side}
	>
		<Tooltip.Arrow class="border-primary-600 rounded-[2px] border-t border-l" />
		{#if tooltip}{@render tooltip()}{:else}
			{text}
		{/if}
	</Tooltip.Content>
</Tooltip.Root>
