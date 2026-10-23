<script lang="ts">
	import type { ClassValue } from 'clsx';
	import type { Snippet } from 'svelte';

	type Props = {
		class?: ClassValue;
		children?: Snippet;
	};

	let { children, class: classProp }: Props = $props();
</script>

<h2
	class={[
		classProp,
		'border-primary-300 text-primary-900 dark:border-primary-600 dark:text-primary-100 mt-4 mb-1 border-b pb-1 text-2xl font-bold'
	]}
>
	{@render children?.()}
</h2>
