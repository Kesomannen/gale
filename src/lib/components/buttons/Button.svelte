<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color: 'green' | 'red' | 'blue' | 'gray';
		label?: string;
		icon?: string;
		children: Snippet;
	} & HTMLButtonAttributes;

	let { color = 'green', class: classes, label, icon, children, ...props }: Props = $props();
</script>

<button
	class="flex items-center gap-1 rounded-lg border-2 border-{color}-500 bg-{color}-700 px-3 py-1.5 text-white duration-100 hover:border-{color}-400 active:border-{color}-600 active:bg-{color}-800 {classes} hover:-translate-y-0.5 active:translate-y-0"
	{...props}
>
	{#if icon}
		<Icon {icon} class="text-lg" />
	{/if}
	{#if label}
		<span>{label}</span>
	{/if}

	{@render children()}
</button>

<style>
	button {
		transition-property: border-color, background-color, transform;
		transition-duration: 90ms;
		transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
	}
</style>
