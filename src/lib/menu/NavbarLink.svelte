<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/stores';
	import Tooltip from '$lib/components/Tooltip.svelte';

	export let to: string;
	export let icon: string;
	export let tooltip: string;

	$: active = $page.url.pathname === to;
</script>

<Tooltip side="right" sideOffset={3} text={tooltip} class="flex">
	<a
		href={to}
		class="relative rounded-lg p-2 text-2xl {active
			? 'text-accent-500 bg-primary-800'
			: 'text-primary-500 hover:bg-primary-800 hover:text-primary-400'}"
	>
		<Icon {icon} />
	</a>
</Tooltip>
