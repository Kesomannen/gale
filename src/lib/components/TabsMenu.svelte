<script lang="ts">
	import { Tabs } from 'bits-ui';

	export let value: string;
	export let options: { value: string; label: string; disabled?: boolean; }[];
</script>

<Tabs.Root bind:value class="overflow-hidden h-full">
	<Tabs.List class="flex gap-1 p-1 my-1 rounded-lg text-slate-300 bg-gray-900">
		{#each options as option}
			<Tabs.Trigger
				value={option.value}
				disabled={option.disabled ?? false}
				class="flex-grow rounded-md px-2 py-0.5 transition-colors duration-75 hover:bg-gray-800 enabled:hover:text-slate-100 data-[state=active]:bg-gray-700 data-[state=active]:text-slate-100 data-[state=active]:font-semibold disabled:cursor-not-allowed disabled:opacity-50"
			>
				{option.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	<slot />
</Tabs.Root>
