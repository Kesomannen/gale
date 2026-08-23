<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	type Props = {
		to: string;
		icon: string;
		tooltip: string;
		disabled?: boolean;
		outline?: boolean;
	};

	let { to, icon, tooltip, outline = true, disabled = false }: Props = $props();

	let active = $derived(page.url.pathname === to);
	let hasOutline = $derived(outline && !active);

	const commonClasses = 'relative flex items-center rounded-lg p-2.5 text-3xl';
</script>

<Tooltip text={tooltip} side="right">
	{#if disabled}
		<button
			disabled
			class={[
				commonClasses,
				'text-primary-300 dark:text-primary-500 cursor-not-allowed opacity-50'
			]}
		>
			{@render icon_()}
		</button>
	{:else}
		<a
			href={to}
			class={[
				active
					? 'text-accent-600 dark:text-accent-500 dark:bg-primary-800 bg-primary-200 font-semibold'
					: 'text-primary-400 hover:text-primary-500 dark:text-primary-500 dark:hover:bg-primary-800 dark:hover:text-primary-400 hover:bg-primary-200',
				commonClasses
			]}
		>
			{@render icon_()}
		</a>
	{/if}
</Tooltip>

{#snippet icon_()}
	<Icon {icon} class={[hasOutline && 'hidden']} />
	<Icon icon="{icon}-outline" class={[!hasOutline && 'hidden']} />
{/snippet}
