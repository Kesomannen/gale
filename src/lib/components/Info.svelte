<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from './Tooltip.svelte';
</script>

<Tooltip class="cursor-help px-2 py-1 text-slate-400" openDelay={50}>
	<Icon icon="mdi:question-mark-circle" />

	<svelte:fragment slot="tooltip">
		<slot />
	</svelte:fragment>
</Tooltip>
