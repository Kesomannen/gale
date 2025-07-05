<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	type Props = {
		to: string;
		icon: string;
		activeIcon: string;
		label: string;
		tooltip: string;
		expanded: boolean;
	};

	let { to, icon, activeIcon, label, tooltip, expanded }: Props = $props();

	let active = $derived(page.url.pathname === to);
</script>

{#snippet link()}
	<a
		href={to}
		class={[
			active
				? 'text-accent-400 bg-primary-800 font-semibold'
				: 'text-primary-600 hover:bg-primary-800 hover:text-primary-400',
			'relative flex items-center gap-2 rounded-md p-2.5'
		]}
	>
		<Icon class="text-[1.75rem]" icon={active ? activeIcon : icon} />

		{#if expanded}
			<span>{label}</span>
		{/if}
	</a>
{/snippet}

{#if expanded}
	{@render link()}
{:else}
	<Tooltip text={tooltip} side="right">
		{@render link()}
	</Tooltip>
{/if}
