<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';

	type Props = {
		to: string;
		icon: string;
		label: string;
		expanded: boolean;
	};

	let { to, icon, label, expanded }: Props = $props();

	let active = $derived(page.url.pathname === to);
</script>

<a
	href={to}
	class={[
		active
			? 'text-accent-400 bg-primary-800 font-semibold'
			: 'text-primary-500 hover:bg-primary-800 hover:text-primary-400 font-medium',
		'relative flex items-center gap-2 rounded-md p-2'
	]}
>
	<Icon class="text-2xl" {icon} />

	{#if expanded}
		<span>{label}</span>
	{/if}
</a>
