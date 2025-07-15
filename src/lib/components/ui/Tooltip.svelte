<script lang="ts">
	import { dropInTo, dropOutFrom } from '$lib/transitions';
	import { Tooltip } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { fade, fly } from 'svelte/transition';

	type Props = {
		text?: string;
		side?: 'top' | 'right' | 'bottom' | 'left';
		sideOffset?: number;
		delayDuration?: number;
		disabled?: boolean;
		class?: string;
		children?: Snippet;
	} & (
		| { text: string; tooltip?: never }
		| {
				text?: never;
				tooltip: Snippet;
		  }
	);

	let {
		text = '',
		side = 'top',
		sideOffset = 0,
		delayDuration = 400,
		disabled = false,
		class: triggerClass,
		children,
		tooltip
	}: Props = $props();

	let distance = $derived(
		{
			top: { x: 0, y: 5 },
			right: { x: -5, y: 0 },
			bottom: { x: 0, y: -5 },
			left: { x: 5, y: 0 }
		}[side]
	);
</script>

<Tooltip.Root {delayDuration}>
	<Tooltip.Trigger class={triggerClass} {disabled}>
		{@render children?.()}
	</Tooltip.Trigger>
	<Tooltip.Content forceMount {sideOffset} {side}>
		{#snippet child({ wrapperProps, props, open })}
			<div {...wrapperProps}>
				{#if open}
					<div
						class="border-primary-600 bg-primary-800 text-primary-300 relative z-50 max-w-lg rounded-lg border px-4 py-2 shadow-md"
						{...props}
						in:fly={dropInTo(distance)}
						out:fade={dropOutFrom(distance)}
					>
						{#if tooltip}{@render tooltip()}{:else}
							{text}
						{/if}
					</div>
				{/if}
			</div>
		{/snippet}
	</Tooltip.Content>
</Tooltip.Root>
