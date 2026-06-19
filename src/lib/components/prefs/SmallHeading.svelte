<script lang="ts">
	import type { ClassValue } from 'clsx';
	import type { Snippet } from 'svelte';

	type Props = {
		class?: ClassValue;
		children?: Snippet;
	};

	let { children, class: classProp }: Props = $props();
</script>

<h3 class={[classProp, 'text-primary-100 mt-4 text-lg font-semibold']}>
	{@render children?.()}
</h3>
