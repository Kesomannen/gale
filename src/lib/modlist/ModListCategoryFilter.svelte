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
		class="bg-primary-900 hover:border-primary-500 flex flex-grow-3 basis-0 items-center overflow-hidden rounded-lg border border-transparent px-3 py-1.5"
	>
		<Icon class="text-primary-400 mr-2 shrink-0 text-lg" {icon} />
		{#if selected.length === 0}
			<span class="text-primary-300 truncate">{label}</span>
		{:else}
			<div class="mr-2 flex flex-wrap gap-1">
				{#each selected as category}
					<div
						class="bg-primary-800 text-primary-200 overflow-hidden rounded-lg py-0.5 pr-0.5 pl-2 text-sm"
					>
						<span class="truncate overflow-hidden">{category}</span>

						<Button.Root
							class="hover:bg-primary-700 ml-0.5 rounded-lg px-1.5"
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
			class="text-primary-400 ml-auto shrink-0 origin-center transform text-lg transition-all duration-100 ease-out {open
				? 'rotate-180'
				: 'rotate-0'}"
			icon="mdi:chevron-down"
		/>
	</Select.Trigger>
</Dropdown>
