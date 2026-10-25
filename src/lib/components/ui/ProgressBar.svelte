<script lang="ts">
	import { Progress } from 'bits-ui';

	type Props = {
		value: number;
		max?: number;
		class?: string;
	};

	let { value, max = 1, class: classProp = '' }: Props = $props();
	let width = $derived(max > 0 ? Math.min((value / max) * 100, 100) : 0);
</script>

<Progress.Root
	{value}
	{max}
	class={[
		classProp,
		'bg-primary-200 dark:bg-primary-700 relative h-2 w-full overflow-hidden rounded-full'
	]}
>
	<div
		class="bg-accent-600 dark:bg-accent-500 absolute top-0 left-0 h-full transition-[width]"
		style:width="{width}%"
	></div>
</Progress.Root>
