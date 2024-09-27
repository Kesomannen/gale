<script lang="ts" generics="T">
	import Icon from '@iconify/svelte';
	import { Select } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import { quartOut, quartIn } from 'svelte/easing';

	type Props<Value> = {
		icon?: string;
		items: Value[];
		selected?: Value | Value[];
		multiple?: boolean;
		placeholder?: string;
		getLabel?: (item: Value) => string;
		class?: string;
	};

	let {
		icon,
		items,
		selected = $bindable(),
		multiple = false,
		placeholder = 'Select an item...',
		getLabel = (item: T) => item as string,
		class: classes = ''
	}: Props<T> = $props();

	let wrappedItems = $derived(items.map((item) => ({ value: item })));
	let wrappedSelection = $derived.by(() => {
		if (selected === undefined) return undefined;
		if (Array.isArray(selected)) {
			return selected.map((value) => ({ value }));
		} else {
			return { value: selected };
		}
	});

	$effect(() => console.log(wrappedSelection));
</script>

<Select.Root
	{multiple}
	items={wrappedItems}
	selected={wrappedSelection}
	onSelectedChange={(newSelected) => {
		if (newSelected === undefined) {
			selected = [];
			return;
		}

		if (Array.isArray(newSelected)) {
			selected = newSelected.map(({ value }) => value);
		} else {
			selected = newSelected.value;
		}
	}}
>
	<Select.Trigger
		class="{classes} flex items-center gap-2 rounded-lg border-2 border-gray-600 bg-gray-700 px-3 py-1.5 text-gray-200 placeholder-gray-300 duration-100 hover:border-gray-500 active:border-gray-600"
	>
		{#if icon}
			<Icon {icon} />
		{/if}
		<Select.Value {placeholder} />
		<Icon icon="material-symbols:expand-all" class="ml-auto" />
	</Select.Trigger>
	<Select.Content
		inTransition={fly}
		inTransitionConfig={{ duration: 100, y: -10, easing: quartOut }}
		outTransition={fly}
		outTransitionConfig={{ duration: 50, y: -20, easing: quartIn }}
		sideOffset={-4}
		sameWidth
		class="content flex max-h-52 flex-col overflow-y-auto rounded-b-lg border-x-2 border-b-2 border-gray-600 bg-gray-700 p-1"
	>
		{#each items as item}
			<Select.Item
				value={item}
				class="flex cursor-pointer items-center rounded px-2 py-1 text-gray-300 hover:bg-gray-600 hover:text-gray-100"
			>
				{getLabel(item)}
				<Select.ItemIndicator class="ml-auto inline text-green-400">
					<Icon icon="mdi:check" />
				</Select.ItemIndicator>
			</Select.Item>
		{/each}
	</Select.Content>
</Select.Root>
