<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from './Tooltip.svelte';
</script>

<Tooltip class="mr-2 cursor-default text-slate-300" openDelay={50}>
	<Icon icon="mdi:info" />

	<svelte:fragment slot="tooltip">
		<slot />
	</svelte:fragment>
</Tooltip>
