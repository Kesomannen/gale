<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/stores';
	import Tooltip from '$lib/components/Tooltip.svelte';

	export let to: string;
	export let icon: string;
	export let tooltip: string;

	$: active = $page.url.pathname === to;
</script>

<Tooltip side="right" text={tooltip} class="flex">
	<a
		href={to}
		class="text-2xl rounded-lg p-2 relative
			{active ? 'text-green-500 bg-gray-800' : 'text-slate-500 hover:bg-gray-800 hover:text-slate-400'}
		"
	>
		<Icon {icon} />

		<!--
		{#if active}
		<div class="absolute right-[-5px] bottom-[-5px] px-1 py-0.5 bg-green-600 text-xs text-slate-100 rounded-md">
			100
		</div>
		{/if}
		-->
	</a>
</Tooltip>
