<script lang="ts">
	import { Tabs } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = {
		value: string;
		options: { value: string; label: string }[];
		children?: Snippet;
	};

	let { value = $bindable(), options, children }: Props = $props();
</script>

<Tabs.Root bind:value>
	<Tabs.List
		class="text-primary-700 dark:bg-primary-900 dark:text-primary-300 bg-primary-100 my-1 flex gap-1 rounded-xl p-1"
	>
		{#each options as option}
			<Tabs.Trigger
				value={option.value}
				class="hover:text-primary-900 dark:hover:bg-primary-800 dark:hover:text-primary-100 dark:data-[state=active]:bg-primary-700 dark:data-[state=active]:text-primary-100 data-[state=active]:text-primary-700 hover:bg-primary-200 data-[state=active]:bg-primary-200 grow rounded-lg px-2 py-0.5 data-[state=active]:font-semibold"
			>
				{option.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	{@render children?.()}
</Tabs.Root>
