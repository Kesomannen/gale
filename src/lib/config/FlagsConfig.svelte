<script lang="ts">
	import { setTaggedConfig } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue, SelectItem } from '$lib/models';
	import { Select } from 'bits-ui';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import { slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';

	export let entryId: ConfigEntryId;

	let open = false;

	let content = entryId.entry.value.content as { indicies: number[]; options: string[] };
	let items = content.options.map(valueToItem);

	let selectedItems = content.indicies.map(indexToItem);

	function indexToItem(index: number): SelectItem {
		return valueToItem(content.options[index]);
	}

	function valueToItem(value: string): SelectItem {
		return { value, label: value };
	}

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { indicies: number[]; options: string[] };
		selectedItems = content.options.map(valueToItem);
	}

	function onSelectedChange(newValues: string[]) {
		content.indicies = newValues.map(value => content.options.indexOf(value));
		setTaggedConfig(entryId, {
			type: 'flags',
			content
		});
	}
</script>

<Select.Root
	{items}
	bind:selected={selectedItems}
	bind:open
	onSelectedChange={(selection) => {
		if (selection) {
			onSelectedChange(selection.map((item) => item.value));
		}
	}}
	multiple={true}
>
	<Select.Trigger
		class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1 text-sm
                border border-gray-500 border-opacity-0 hover:border-opacity-100 overflow-hidden"
	>
		<Select.Value class="text-slate-300 text-left w-full truncate" />
		<Icon
			class="text-slate-400 text-xl ml-auto transition-all
                   transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
			icon="mdi:chevron-down"
		/>
	</Select.Trigger>
	<Select.Content
		class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
		transition={slide}
		transitionConfig={{ duration: 100 }}
	>
		{#each items as item}
			<Select.Item
				value={item.value}
				label={item.label}
				class="flex items-center px-3 py-1 truncate text-sm text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
			>
				{item.label}
				<Select.ItemIndicator class="ml-auto">
					<Icon icon="mdi:check" class="text-green-400 text-lg" />
				</Select.ItemIndicator>
			</Select.Item>
		{/each}
	</Select.Content>
</Select.Root>
<ResetConfigButton {entryId} {onReset} />
