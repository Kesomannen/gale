<script lang="ts">
	type Props = {
		href?: string;
		children?: import('svelte').Snippet;
	};

	let { href = '', children }: Props = $props();
	export const raw = false;
	export const title = '';
	export const text = '';
</script>

<a {href} target="_blank">
	{@render children?.()}
</a>
