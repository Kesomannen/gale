<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h3 class="text-primary-100 mt-2 text-lg font-semibold">{@render children?.()}</h3>
