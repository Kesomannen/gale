<script lang="ts">
	type Props = {
		children?: import('svelte').Snippet;
	};

	let { children }: Props = $props();
</script>

<label class="text-primary-300 w-[35%] min-w-52 cursor-default truncate text-left">
	{@render children?.()}
</label>
