<script lang="ts">
	import type { ConfigEntryId, ConfigValue, SelectItem } from '$lib/models';
	import { Select } from 'bits-ui';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import { slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { setTaggedConfig } from '$lib/invoke';

	export let entryId: ConfigEntryId;

	let open = false;

	let content = entryId.entry.value.content as { index: number; options: string[] };
	let items = content.options.map(valueToItem);

	let selectedItem = indexToItem(content.index);

	function indexToItem(index: number): SelectItem {
		return valueToItem(content.options[index]);
	}

	function valueToItem(value: string): SelectItem {
		return { value, label: value };
	}

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { index: number; options: string[] };
		selectedItem = indexToItem(content.index);
	}

	function onSelectChange(value: string) {
		let index = content.options.indexOf(value);
		setTaggedConfig(entryId, {
			type: 'enum',
			content: {
				index,
				options: content.options
			}
		});
	}
</script>

<Select.Root
	{items}
	bind:selected={selectedItem}
	bind:open
	onSelectedChange={(selected) => {
		if (selected) {
			onSelectChange(selected.value);
		}
	}}
>
	<Select.Trigger
		class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1 text-sm
                border border-gray-500 border-opacity-0 hover:border-opacity-100"
	>
		<Select.Value class="text-slate-300 text-left w-full" />
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
