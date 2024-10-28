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
		class="flex flex-grow-[3] basis-0 items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 py-1.5 hover:border-opacity-100"
	>
		<Icon class="mr-2 flex-shrink-0 text-lg text-gray-400" {icon} />
		{#if selected.length === 0}
			<span class="truncate text-gray-300">{label}</span>
		{:else}
			<div class="mr-2 flex flex-wrap gap-1">
				{#each selected as category}
					<div
						class="overflow-hidden rounded-lg bg-gray-800 py-0.5 pl-2 pr-0.5 text-sm text-gray-200"
					>
						<span class="overflow-hidden truncate">{category}</span>

						<Button.Root
							class="ml-0.5 rounded-lg px-1.5 hover:bg-gray-700"
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
			class="ml-auto flex-shrink-0 origin-center transform text-lg text-gray-400 transition-all duration-100 ease-out {open
				? 'rotate-180'
				: 'rotate-0'}"
			icon="mdi:chevron-down"
		/>
	</Select.Trigger>
</Dropdown>
