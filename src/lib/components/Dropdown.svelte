<script lang="ts" generics="T">
	import { t, tArray } from '$i18n';

	import { expoOut, quadOut, quartOut } from 'svelte/easing';

	import { Select } from 'bits-ui';
	import { fly, slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';

	export let items: T[];
	export let selected: T | T[] = [];
	export let open = false;
	export let multiple = false;
	export let avoidCollisions = true;
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let placeholder: string = '';
	export let onSelectedChange = (items: T[]) => {};
	export let onSelectedChangeSingle = (item: T) => {};
	export let getLabel = (item: T) => item as string;
	export let EnableTransitions = false;

	let className: string = '';

	export { className as class };

	$: stringValue = Array.isArray(selected)
		? selected.length > 0
			? selected.map((item) => getLabel(item)).join(', ')
			: placeholder
		: getLabel(selected);

	let stringValueText = EnableTransitions ? (Array.isArray(selected) ? tArray(stringValue) : t(stringValue)) : stringValue;
</script>

<Select.Root
	items={items.map((item) => ({ value: item }))}
	onSelectedChange={(selection) => {
		if (!selection) return;
		let values = Array.isArray(selection)
			? selection.map((single) => single.value)
			: [selection.value];

		onSelectedChange(values);

		if (multiple) {
			selected = values;
		} else {
			onSelectedChangeSingle(values[0]);
			selected = values[0];
		}
	}}
	selected={Array.isArray(selected)
		? selected.map((item) => ({ value: item }))
		: { value: selected }}
	{multiple}
	bind:open
>
	<slot name="trigger" text={stringValueText} {open}>
		<Select.Trigger
			class="flex items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 py-1 pl-3 pr-2 hover:border-opacity-100 {className}"
		>
			<div class="flex-shrink flex-grow truncate text-left text-slate-300 text-{size}">
				{stringValueText}
			</div>
			<Icon
				class="flex-shrink-0 origin-center transform text-xl text-slate-400 transition-all duration-100 ease-out {open
					? 'rotate-180'
					: 'rotate-0'}"
				icon="mdi:chevron-down"
			/>
		</Select.Trigger>
	</slot>
	<Select.Content
		class="flex max-h-96 flex-col gap-0.5 overflow-y-auto rounded-lg border border-gray-600 bg-gray-800 p-1 shadow-xl"
		transition={slide}
		transitionConfig={{ duration: 75, easing: quadOut }}
		{avoidCollisions}
	>
		{#each items as item}
			<slot name="item" {item}>
				<Select.Item
					value={item}
					class="flex items-center rounded-md px-3 py-1 text-left text-slate-400 text-{size} cursor-default hover:bg-gray-700 hover:text-slate-200"
				>
					{EnableTransitions ? t(getLabel(item)) : getLabel(item)}
					<Select.ItemIndicator class="ml-auto">
						<Icon icon="mdi:check" class="text-lg text-green-400" />
					</Select.ItemIndicator>
				</Select.Item>
			</slot>
		{/each}
	</Select.Content>
</Select.Root>
