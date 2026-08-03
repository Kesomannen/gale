<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	type Props = {
		to: string;
		icon: string;
		tooltip: string;
		outline?: boolean;
	};

	let { to, icon, tooltip, outline = true }: Props = $props();

	let active = $derived(page.url.pathname === to);
	let hasOutline = $derived(outline && !active);
</script>

<Tooltip text={tooltip} side="right">
	<a
		href={to}
		class={[
			active
				? 'text-accent-500 bg-primary-800 font-semibold'
				: 'text-primary-500 hover:bg-primary-800 hover:text-primary-400',
			'relative flex items-center rounded-lg p-2.5 text-3xl'
		]}
	>
		<Icon {icon} class={[hasOutline && 'hidden']} />
		<Icon icon="{icon}-outline" class={[!hasOutline && 'hidden']} />
	</a>
</Tooltip>
