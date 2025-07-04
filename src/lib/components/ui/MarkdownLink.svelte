<script lang="ts">
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	let { children, target, ...props }: HTMLAnchorAttributes = $props();
</script>

<a {...props} target="_blank">
	{@render children?.()}
</a>
