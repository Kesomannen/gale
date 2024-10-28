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
			? 'bg-gray-800 text-green-500'
			: 'text-gray-500 hover:bg-gray-800 hover:text-gray-400'}"
	>
		<Icon {icon} />
	</a>
</Tooltip>
