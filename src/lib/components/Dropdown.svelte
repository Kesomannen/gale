<script lang="ts" generics="T">
	import { expoOut, quadOut, quartOut } from 'svelte/easing';

	import { Select } from 'bits-ui';
	import { fly, slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { sentenceCase } from '$lib/util';

	export let items: T[];
	export let selected: T | T[];
	export let open = false;
	export let multiple = false;
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let onSelectedChange = (items: T[]) => {};
	export let onSelectedChangeSingle = (item: T) => {};
	export let getLabel: (item: T) => string = (item) => sentenceCase(item as string);

	let className: string = '';

	export { className as class };

	$: stringValue = Array.isArray(selected)
		? selected.map((item) => getLabel(item)).join(', ')
		: getLabel(selected);
</script>

<Select.Root
	items={items.map((item) => ({ value: item }))}
	onSelectedChange={(selection) => {
		if (!selection) return;
		let values = Array.isArray(selection)
			? selection.map((single) => single.value)
			: [selection.value];

		onSelectedChange(values);

		if (values.length === 1) {
			onSelectedChangeSingle(values[0]);
			selected = values[0];
		} else {
			selected = values;
		}
	}}
	selected={Array.isArray(selected)
		? selected.map((item) => ({ value: item }))
		: { value: selected }}
	{multiple}
	bind:open
>
	<slot name="trigger" text={stringValue} {open}>
		<Select.Trigger
			class="flex items-center overflow-hidden bg-gray-900 rounded-lg pl-3 pr-2 py-1
            border border-gray-500 border-opacity-0 hover:border-opacity-100 {className}"
		>
			<div class="text-slate-300 text-left flex-grow flex-shrink truncate text-{size}">
				{stringValue}
			</div>
			<Icon
				class="text-slate-400 text-xl transition-all flex-shrink-0 duration-100 ease-out
                transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
				icon="mdi:chevron-down"
			/>
		</Select.Trigger>
	</slot>
	<Select.Content
		class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-48 rounded-lg border border-gray-600"
		transition={slide}
		transitionConfig={{ duration: 75, easing: quadOut }}
	>
		{#each items as item}
			<slot name="item" {item}>
				<Select.Item
					value={item}
					class="flex items-center px-3 py-1 text-slate-400 text-left rounded-md text-{size}
                hover:bg-gray-700 hover:text-slate-200 cursor-default"
				>
					{getLabel(item)}

					<Select.ItemIndicator class="ml-auto">
						<Icon icon="mdi:check" class="text-green-400 text-lg" />
					</Select.ItemIndicator>
				</Select.Item>
			</slot>
		{/each}
	</Select.Content>
</Select.Root>
