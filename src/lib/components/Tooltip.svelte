<script lang="ts">
	import { Tooltip } from 'bits-ui';
	import { quadOut } from 'svelte/easing';
	import { fade, fly } from 'svelte/transition';

	export let text: string = '[No text provided]';
	export let side: 'top' | 'right' | 'bottom' | 'left' = 'top';
	export let sideOffset: number = 0;
	export let openDelay: number = 150;

	let triggerClass: string = '';

	const flyDistances = {
		top: { x: 0, y: 7 },
		right: { x: -7, y: 0 },
		bottom: { x: 0, y: -7 },
		left: { x: 7, y: 0 }
	};

	$: flyDistance = flyDistances[side];

	export { triggerClass as class };
</script>

<Tooltip.Root {openDelay}>
	<Tooltip.Trigger class={triggerClass}>
		<slot />
	</Tooltip.Trigger>
	<Tooltip.Content
		class="max-w-[40rem] rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-slate-300 shadow-lg"
		inTransition={fly}
		inTransitionConfig={{ duration: 100, ...flyDistance, easing: quadOut }}
		outTransition={fade}
		outTransitionConfig={{ duration: 100 }}
		{sideOffset}
		{side}
	>
		<Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
		<slot name="tooltip">
			{text}
		</slot>
	</Tooltip.Content>
</Tooltip.Root>
