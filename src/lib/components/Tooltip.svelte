<script lang="ts">
	import { Tooltip } from 'bits-ui';
	import { fade } from 'svelte/transition';

	export let text: string = '';
	export let side: 'top' | 'right' | 'bottom' | 'left' = 'top';
	export let sideOffset: number = 0;
	export let openDelay: number = 100;

	let triggerClass: string = '';

	export { triggerClass as class };
</script>

<Tooltip.Root {openDelay}>
	<Tooltip.Trigger class={triggerClass}>
		<slot />
	</Tooltip.Trigger>
	<Tooltip.Content
		class="max-w-[40rem] rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-slate-300 shadow-lg"
		transition={fade}
		transitionConfig={{ duration: 50 }}
		{sideOffset}
		{side}
	>
		<Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
		<slot name="tooltip">
			{text}
		</slot>
	</Tooltip.Content>
</Tooltip.Root>
