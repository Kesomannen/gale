<script lang="ts">
	import { Tabs } from 'bits-ui';

	export let value: string;
	export let options: { value: string; label: string }[];
</script>

<Tabs.Root bind:value>
	<Tabs.List class="my-1 flex gap-1 rounded-xl bg-slate-900 p-1 text-slate-300">
		{#each options as option}
			<Tabs.Trigger
				value={option.value}
				class="grow rounded-lg px-2 py-0.5 hover:bg-slate-800 hover:text-slate-100 data-[state=active]:bg-slate-700 data-[state=active]:font-semibold data-[state=active]:text-slate-100"
			>
				{option.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	<slot />
</Tabs.Root>
