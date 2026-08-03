<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		label?: string;
	} & HTMLButtonAttributes;

	let { label, class: classProp, ...restProps }: Props = $props();
</script>

<button
	{...restProps}
	class={[
		classProp,
		'text-primary-400 disabled:text-primary-500 enabled:hover:bg-primary-700 enabled:hover:text-primary-300 flex items-center gap-2 rounded-md p-1.5 disabled:cursor-not-allowed'
	]}
>
	<Icon icon="mdi:refresh" class="text-lg" />

	{#if label}
		<span class="text-sm">{label}</span>
	{/if}
</button>
