<script lang="ts">
	import Tooltip from './Tooltip.svelte';

	export let text: string;
</script>

<Tooltip class="w-[30%] min-w-52 cursor-default truncate text-left text-slate-300">
	{text}
	<svelte:fragment slot="tooltip">
		<slot />
	</svelte:fragment>
</Tooltip>
