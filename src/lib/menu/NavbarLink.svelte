<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/stores';
	import Tooltip from '$lib/components/Tooltip.svelte';

	export let to: string;
	export let icon: string;
	export let tooltip: string;

	$: active = $page.url.pathname === to;
</script>

<Tooltip side="right" sideOffset={3} text={tooltip} class="relative flex">
	<a
		href={to}
		class="relative rounded-lg p-2 text-2xl {active
			? 'text-accent-500 bg-slate-800'
			: 'text-slate-500 hover:bg-slate-800 hover:text-slate-400'}"
	>
		<Icon {icon} />
	</a>
</Tooltip>
