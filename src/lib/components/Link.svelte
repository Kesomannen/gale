<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		href: string;
		children?: Snippet;
	};

	let { href, children }: Props = $props();
</script>

<a
	{href}
	target="_blank"
	rel="noopener noreferrer"
	class="text-accent-400 hover:text-accent-300 hover:underline"
>
	{@render children?.()}
</a>
