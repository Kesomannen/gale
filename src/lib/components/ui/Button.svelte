<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color?: 'accent' | 'primary' | 'red';
		size?: 'md' | 'lg';
		icon?: string;
		loading?: boolean;
	} & HTMLButtonAttributes;

	let {
		disabled: disabledProp,
		color = 'accent',
		size = 'md',
		icon,
		loading = false,
		class: classProp,
		children,
		...restProps
	}: Props = $props();

	let typeClass = $derived(
		{
			accent:
				'dark:enabled:hover:bg-accent-600 dark:bg-accent-700 enabled:hover:bg-accent-500 bg-accent-600 font-medium text-white',
			primary:
				'enabled:hover:bg-primary-300 bg-primary-200 text-primary-700 dark:enabled:hover:bg-primary-600 dark:bg-primary-700 dark:text-primary-200',
			red: 'enabled:hover:bg-red-600 bg-red-700 text-white'
		}[color]
	);
	let sizeClasses = $derived(
		{
			md: 'text-base px-4 py-2',
			lg: 'text-lg px-6 py-2.5 font-medium'
		}[size]
	);

	let disabled = $derived(disabledProp || loading);
	let renderedIcon = $derived(loading ? 'ph:circle-notch' : icon);
</script>

<button
	class={[
		classProp,
		typeClass,
		sizeClasses,
		'disabled:text-primary-500 dark:disabled:bg-primary-700 dark:disabled:text-primary-400 disabled:bg-primary-300 inline-flex shrink-0 items-center justify-center overflow-hidden rounded-lg text-nowrap disabled:cursor-not-allowed disabled:opacity-70'
	]}
	{disabled}
	{...restProps}
>
	{#if renderedIcon}
		<Icon
			icon={renderedIcon}
			class={['mr-2', size === 'lg' ? 'text-xl' : 'text-lg', loading && 'animate-spin']}
		/>
	{/if}

	{@render children?.()}
</button>
