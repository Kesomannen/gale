<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<label class="text-primary-300 w-[35%] min-w-52 cursor-default truncate text-left">
	{@render children?.()}
</label>
