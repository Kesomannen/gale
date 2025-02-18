<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import { categories } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { Button, Select } from 'bits-ui';

	export let selected: string[];
	export let excluded: string[];
	export let icon: string;
	export let label: string;
</script>

<Dropdown
	items={$categories
		.map(({ name }) => name)
		.filter((category) => !excluded.includes(category))
		.toSorted()}
	multiple={true}
	bind:selected
>
	<Select.Trigger
		let:open
		slot="trigger"
		class="flex flex-grow-3 basis-0 items-center overflow-hidden rounded-lg border border-transparent bg-slate-900 px-3 py-1.5 hover:border-slate-500"
	>
		<Icon class="mr-2 shrink-0 text-lg text-slate-400" {icon} />
		{#if selected.length === 0}
			<span class="truncate text-slate-300">{label}</span>
		{:else}
			<div class="mr-2 flex flex-wrap gap-1">
				{#each selected as category}
					<div
						class="overflow-hidden rounded-lg bg-slate-800 py-0.5 pr-0.5 pl-2 text-sm text-slate-200"
					>
						<span class="truncate overflow-hidden">{category}</span>

						<Button.Root
							class="ml-0.5 rounded-lg px-1.5 hover:bg-slate-700"
							on:click={(evt) => {
								evt.stopPropagation();
								selected = selected.filter((cat) => cat !== category);
							}}
						>
							x
						</Button.Root>
					</div>
				{/each}
			</div>
		{/if}
		<Icon
			class="ml-auto shrink-0 origin-center transform text-lg text-slate-400 transition-all duration-100 ease-out {open
				? 'rotate-180'
				: 'rotate-0'}"
			icon="mdi:chevron-down"
		/>
	</Select.Trigger>
</Dropdown>
