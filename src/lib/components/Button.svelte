<script lang="ts" module>
	export type Weight = 'primary' | 'secondary' | 'tertiary';
	export type Color = 'green' | 'red' | 'blue' | 'gray';
</script>

<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type WeightProp =
		| {}
		| { primary: true }
		| { secondary: true }
		| { tertiary: true }
		| { weight: Weight };

	type Props = {
		color?: Color;
		label?: string;
		icon?: string;
		href?: string;
		children?: Snippet;
	} & WeightProp &
		HTMLButtonAttributes;

	let { color = 'green', class: classes, label, icon, href, children, ...props }: Props = $props();

	const colors = {
		green: {
			primary:
				'text-white border-green-500 bg-green-700 hover:border-green-400 active:border-green-600 active:bg-green-800',
			secondary: 'text-green-400 border-green-500 hover:border-green-400 active:border-green-600',
			tertiary: 'text-green-400 border-green-500/0 active:text-green-500'
		},
		red: {
			primary:
				'text-white border-red-500 bg-red-700 hover:border-red-400 active:border-red-600 active:bg-red-800',
			secondary: 'text-red-400 border-red-500 hover:border-red-400 active:border-red-600',
			tertiary: 'text-red-400 border-red-500/0 active:text-red-500'
		},
		blue: {
			primary:
				'text-white border-blue-500 bg-blue-700 hover:border-blue-400 active:border-blue-600 active:bg-blue-800',
			secondary: 'text-blue-400 border-blue-500 hover:border-blue-400 active:border-blue-600',
			tertiary: 'text-blue-400 border-blue-500/0 active:text-blue-500'
		},
		gray: {
			primary:
				'text-gray-300 border-gray-500 bg-gray-700 hover:border-gray-400 active:border-gray-500 active:bg-gray-800',
			secondary: 'text-gray-300 border-gray-500 hover:border-gray-400 active:border-gray-500',
			tertiary: 'text-gray-300 border-gray-400/0 active:text-gray-400'
		}
	};

	let weight: Weight = $derived.by(() => {
		if ('primary' in props) return 'primary';
		if ('secondary' in props) return 'secondary';
		if ('tertiary' in props) return 'tertiary';
		if ('weight' in props) return props.weight;
		return 'primary';
	});

	let colorClasses = $derived(colors[color][weight]);
</script>

<Button.Root
	class="btn {colorClasses} flex items-center justify-center gap-1 rounded-lg border-2 px-3 py-1.5 duration-100 disabled:border-gray-500 disabled:bg-gray-700 disabled:text-gray-300 {classes} enabled:hover:translate-y-[-1px] enabled:active:translate-y-0 disabled:cursor-not-allowed disabled:opacity-75"
	{...props}
>
	{#if icon}
		<Icon {icon} class="text-lg" />
	{/if}
	{#if label}
		<span>{label}</span>
	{/if}

	{@render children?.()}
</Button.Root>

<style lang="postcss">
	.btn {
		transition-property: border-color, background-color, transform;
		transition-timing-function: cubic-bezier(0.19, 1, 0.22, 1);
	}
</style>
