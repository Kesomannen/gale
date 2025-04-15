<script lang="ts">
	import { dropTransitionTo } from '$lib/transitions';
	import { Tooltip } from 'bits-ui';

	export let text: string = '[No text provided]';
	export let side: 'top' | 'right' | 'bottom' | 'left' = 'top';
	export let sideOffset: number = 0;
	export let openDelay: number = 150;

	let triggerClass: string = '';

	const distances = {
		top: { x: 0, y: 7 },
		right: { x: -7, y: 0 },
		bottom: { x: 0, y: -7 },
		left: { x: 7, y: 0 }
	};

	$: distance = distances[side];

	export { triggerClass as class };
</script>

<Tooltip.Root {openDelay}>
	<Tooltip.Trigger class={triggerClass}>
		<slot />
	</Tooltip.Trigger>
	<Tooltip.Content
		class="border-primary-600 bg-primary-800 text-primary-300 max-w-lg cursor-help rounded-lg border px-4 py-2 shadow-lg"
		{...dropTransitionTo(distance)}
		{sideOffset}
		{side}
	>
		<Tooltip.Arrow class="border-primary-600 rounded-[2px] border-t border-l" />
		<slot name="tooltip">
			{text}
		</slot>
	</Tooltip.Content>
</Tooltip.Root>
