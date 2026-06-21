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
				? 'text-accent-500 bg-primary-800 font-semibold'
				: 'text-primary-500 hover:bg-primary-800 hover:text-primary-400',
			'flex items-center rounded-lg p-3 text-3xl'
		]}
	>
		<!-- Keep both icons in the DOM to avoid layout shifts when the filled icon loads. -->
		<Icon {icon} class={[active && 'hidden']} />
		<Icon icon="{icon}-fill" class={[!active && 'hidden']} />
	</a>
</Tooltip>
