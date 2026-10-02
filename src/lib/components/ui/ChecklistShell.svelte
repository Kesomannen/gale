<script lang="ts">
	import type { ClassValue } from 'clsx';
	import type { Snippet } from 'svelte';
	import Checkbox from './Checkbox.svelte';

	type Props = {
		class?: ClassValue;
		title?: string;
		allIncluded: boolean;
		anyIncluded?: boolean;
		onToggleAll: (value: boolean) => void;
		children: Snippet;
	};

	let {
		class: classProp,
		title,
		allIncluded,
		anyIncluded = false,
		onToggleAll,
		children
	}: Props = $props();
</script>

<div
	class={[
		classProp,
		'border-primary-200 dark:border-primary-900 relative flex flex-col overflow-hidden rounded-lg border'
	]}
>
	{#if title}
		<label
			class="text-primary-900 dark:bg-primary-900 bg-primary-100 flex w-full shrink-0 items-center px-4 py-2.5 font-medium dark:text-white"
		>
			<Checkbox
				class="mr-3"
				checked={allIncluded}
				indeterminate={anyIncluded && !allIncluded}
				onCheckedChange={onToggleAll}
			/>
			{title}
		</label>
	{/if}

	{@render children()}
</div>
