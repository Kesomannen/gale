<script lang="ts">
	import { Button } from "bits-ui";

  export let disabled: boolean = false;
  export let color: string = "green";
  export let outline: boolean = false;
  export let fontWeight: string = "normal";
  export let onClick: ((evt: MouseEvent) => void) | undefined = undefined;
</script>

<Button.Root
	class="rounded-lg font-{fontWeight} px-4 py-2 text-slate-100 bg-{color}-700 hover:bg-{color}-600
        {outline && `border-${color}-600 bg-opacity-0 border`}
        disabled:bg-gray-600/80 disabled:hover:bg-gray-600/80 disabled:text-gray-200/80 disabled:cursor-not-allowed"
	on:click={onClick}
  bind:disabled
>
	<slot />
</Button.Root>
