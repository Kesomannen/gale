<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h2 class="border-primary-500 text-primary-100 mt-4 mb-1 border-b pb-1 text-2xl font-bold">
	{@render children?.()}
</h2>
