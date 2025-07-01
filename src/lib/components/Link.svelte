<script lang="ts">
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	type Props = Omit<HTMLAnchorAttributes, 'target' | 'rel'>;

	let { class: classProp, children, ...props }: Props = $props();
</script>

<a
	target="_blank"
	rel="noopener noreferrer"
	class={[classProp, 'text-accent-400 hover:text-accent-300 cursor-pointer hover:underline']}
	{...props}
>
	{@render children?.()}
</a>
