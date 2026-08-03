<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { ClassValue } from 'clsx';
	import type { Snippet } from 'svelte';

	type Props = {
		icon?: string;
		title: string;
		class?: ClassValue;
		children?: Snippet;
	};

	let { icon, title, class: classProp, children }: Props = $props();
</script>

<div class={[classProp, 'text-primary-400 flex flex-col items-center justify-center text-center']}>
	{#if icon}
		<Icon {icon} class="text-primary-500 mx-auto mt-4 text-9xl" />
	{/if}

	<div class="text-primary-300 mt-1 text-lg">{title}</div>
	{@render children?.()}
</div>
