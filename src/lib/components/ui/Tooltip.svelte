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
		delayDuration = 150,
		disabled = false,
		class: triggerClass = '',
		children,
		tooltip
	}: Props = $props();

	let distance = $derived(
		{
			top: { x: 0, y: 7 },
			right: { x: -7, y: 0 },
			bottom: { x: 0, y: -7 },
			left: { x: 7, y: 0 }
		}[side]
	);
</script>

<Tooltip.Root {delayDuration}>
	<Tooltip.Trigger class={triggerClass} {disabled}>
		{@render children?.()}
	</Tooltip.Trigger>
	<Tooltip.Content forceMount {sideOffset} {side}>
		{#snippet child({ wrapperProps, props, open })}
			{#if open}
				<div {...wrapperProps}>
					<div
						class="border-primary-600 bg-primary-800 text-primary-300 relative max-w-lg rounded-lg border px-4 py-2 shadow-lg"
						{...props}
						in:fly={dropInTo(distance)}
						out:fade={dropOutFrom(distance)}
					>
						{#if tooltip}{@render tooltip()}{:else}
							{text}
						{/if}
					</div>
				</div>
			{/if}
		{/snippet}
	</Tooltip.Content>
</Tooltip.Root>
