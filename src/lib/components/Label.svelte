<script lang="ts">
	import Tooltip from "./Tooltip.svelte";

  export let text: string;
</script>

<Tooltip class="text-slate-300 min-w-52 truncate cursor-default text-left">
  {text}
  <svelte:fragment slot="tooltip">
    <slot />
  </svelte:fragment>
</Tooltip>