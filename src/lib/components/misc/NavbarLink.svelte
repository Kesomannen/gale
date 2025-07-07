<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	type Props = {
		to: string;
		icon: string;
		tooltip: string;
	};

	let { to, icon, tooltip }: Props = $props();

	let active = $derived(page.url.pathname === to);
</script>

<Tooltip text={tooltip} side="right">
	<a
		href={to}
		class={[
			active
				? 'text-accent-400 bg-primary-800 font-semibold'
				: 'text-primary-600 hover:bg-primary-800 hover:text-primary-400',
			'relative flex items-center gap-2 rounded-lg p-2.5'
		]}
	>
		<Icon class="text-[1.75rem]" {icon} />
	</a>
</Tooltip>
