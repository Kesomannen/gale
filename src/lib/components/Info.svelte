<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from './Tooltip.svelte';
</script>

<Tooltip class="text-primary-400 cursor-help px-2 py-1" openDelay={50}>
	<Icon icon="mdi:question-mark-circle" />

	<svelte:fragment slot="tooltip">
		<slot />
	</svelte:fragment>
</Tooltip>
