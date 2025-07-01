<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color?: 'accent' | 'primary' | 'red';
		fontWeight?: 'normal' | 'medium' | 'semibold';
		class?: string;
		children?: Snippet;
	} & HTMLButtonAttributes;

	let {
		disabled,
		color = 'accent',
		fontWeight = 'normal',
		class: classProp = '',
		children,
		...props
	}: Props = $props();

	let fontClasses = $derived(
		{
			normal: 'font-normal',
			medium: 'font-medium',
			semibold: 'font-semibold'
		}[fontWeight]
	);

	let colorClasses = $derived(
		{
			accent: 'enabled:hover:bg-accent-600 bg-accent-700',
			primary: 'enabled:hover:bg-primary-600 bg-primary-700',
			red: 'enabled:hover:bg-red-600 bg-red-700'
		}[color]
	);
</script>

<button
	class={[
		classProp,
		fontClasses,
		colorClasses,
		'disabled:opactiy-70 disabled:bg-primary-700 disabled:text-primary-400 inline-flex items-center overflow-hidden rounded-lg px-4 py-2 text-nowrap text-white disabled:cursor-not-allowed'
	]}
	{...props}
>
	{@render children?.()}
</button>
