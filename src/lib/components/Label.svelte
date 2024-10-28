<script lang="ts">
	import Tooltip from './Tooltip.svelte';

	export let text: string;
</script>

<Tooltip class="w-[35%] min-w-52 cursor-default truncate text-left text-gray-300">
	{text}

	<svelte:fragment slot="tooltip">
		<slot />
	</svelte:fragment>
</Tooltip>
