<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from './Tooltip.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<Tooltip class="text-primary-400 cursor-help px-2 py-1" delayDuration={100}>
	<Icon icon="mdi:question-mark-circle" />

	{#snippet tooltip()}
		{@render children?.()}
	{/snippet}
</Tooltip>
