<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color?: 'accent' | 'primary' | 'red';
		icon?: string;
		loading?: boolean;
	} & HTMLButtonAttributes;

	let {
		disabled: disabledProp,
		color = 'accent',
		icon,
		loading = false,
		class: classProp,
		children,
		...restProps
	}: Props = $props();

	let typeClass = $derived(
		{
			accent: 'enabled:hover:bg-accent-600 bg-accent-700 font-medium text-black',
			primary: 'enabled:hover:bg-primary-600 bg-primary-700 text-primary-200',
			red: 'enabled:hover:bg-red-600 bg-red-700 text-black'
		}[color]
	);

	let disabled = $derived(disabledProp || loading);
	let renderedIcon = $derived(loading ? 'mdi:loading' : icon);
</script>

<button
	class={[
		classProp,
		typeClass,
		'disabled:opactiy-70 disabled:bg-primary-700 disabled:text-primary-400 inline-flex items-center overflow-hidden rounded-lg px-4 py-2 text-nowrap disabled:cursor-not-allowed'
	]}
	{disabled}
	{...restProps}
>
	{#if renderedIcon}
		<Icon icon={renderedIcon} class="mr-2 text-lg {loading && 'animate-spin'}" />
	{/if}

	{@render children?.()}
</button>
