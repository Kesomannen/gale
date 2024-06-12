<script lang="ts">
	import { Tooltip } from "bits-ui";
	import { fly } from "svelte/transition";

	export let text: string = ""
	export let side: "top" | "right" | "bottom" | "left" = "top";
	export let sideOffset: number = 0;
	export let openDelay: number = 150;

	let triggerClass: string = "";

  export { triggerClass as class };
</script>

<Tooltip.Root {openDelay}>
	<Tooltip.Trigger class={triggerClass}>
		<slot />
	</Tooltip.Trigger>
	<Tooltip.Content
		class="rounded-lg bg-gray-800 border border-gray-600 text-slate-300 px-4 py-2 shadow-lg max-w-[35rem]"
		transition={fly}
		transitionConfig={{ duration: 100 }}
		{sideOffset}
		{side}
	>
		<Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
		<slot name="tooltip">
      {text}
    </slot>
	</Tooltip.Content>
</Tooltip.Root>