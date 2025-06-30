<script lang="ts">
	import { Tabs } from 'bits-ui';

	type Props = {
		value: string;
		options: { value: string; label: string }[];
		children?: import('svelte').Snippet;
	};

	let { value = $bindable(), options, children }: Props = $props();
</script>

<Tabs.Root bind:value>
	<Tabs.List class="bg-primary-900 text-primary-300 my-1 flex gap-1 rounded-xl p-1">
		{#each options as option}
			<Tabs.Trigger
				value={option.value}
				class="hover:bg-primary-800 hover:text-primary-100 data-[state=active]:bg-primary-700 data-[state=active]:text-primary-100 grow rounded-lg px-2 py-0.5 data-[state=active]:font-semibold"
			>
				{option.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	{@render children?.()}
</Tabs.Root>
