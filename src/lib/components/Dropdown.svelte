<!-- @migration-task Error while migrating Svelte code: This migration would change the name of a slot (item to item_1) making the component unusable -->
<script lang="ts" generics="T, Multiple extends boolean = false">
	import { dropTransition } from '$lib/transitions';

	import { Select } from 'bits-ui';
	import Icon from '@iconify/svelte';

	type Selection = Multiple extends true ? T[] : T;

	export let items: T[];
	export let selected: Selection;
	export let onSelectedChange = (value: Selection) => {};

	export let open = false;
	export let disabled = false;
	export let multiple: Multiple;

	export let icon: string | null = null;
	export let overrideLabel: string | null = null;
	export let placeholder: string = '';
	export let getLabel = (item: T) => item as string;

	export let avoidCollisions = true;

	let className: string = '';

	export { className as class };

	$: label = Array.isArray(selected)
		? selected.length > 0
			? selected.map((item) => getLabel(item)).join(', ')
			: null
		: getLabel(selected as T);

	type BitsSelection = { value: T; label?: string };

	function onSelectionChangeHandler(selection: BitsSelection | BitsSelection[] | undefined) {
		if (selection === undefined) return;

		if (multiple) {
			let values = selection as BitsSelection[];
			selected = values.map(({ value }) => value) as Selection;
		} else {
			let { value } = selection as BitsSelection;
			selected = value as Selection;
		}

		onSelectedChange(selected);
	}

	$: bitsSelected = Array.isArray(selected)
		? (selected as T[]).map((value) => ({
				value
			}))
		: { value: selected as T };
</script>

<Select.Root
	items={items.map((item) => ({ value: item }))}
	onSelectedChange={onSelectionChangeHandler}
	selected={bitsSelected}
	{disabled}
	{multiple}
	bind:open
>
	<slot name="trigger" text={label} {open}>
		<Select.Trigger
			class="group bg-primary-900 enabled:hover:border-primary-500 flex items-center gap-2 overflow-hidden rounded-lg border border-transparent py-1 pr-2 pl-3 disabled:cursor-not-allowed {className}"
		>
			{#if icon}
				<Icon class="text-primary-400 shrink-0 text-lg" {icon} />
			{/if}

			<div
				class="text-primary-300 group-disabled:text-primary-400 shrink grow truncate text-left"
				class:text-primary-300={overrideLabel || label}
				class:text-primary-400={!overrideLabel && !label}
			>
				{overrideLabel ?? label ?? placeholder}
			</div>

			<Icon
				class="text-primary-400 group-disabled:text-primary-500 shrink-0 origin-center transform text-lg transition-all duration-100 ease-out {open
					? 'rotate-180'
					: 'rotate-0'}"
				icon="mdi:chevron-down"
			/>
		</Select.Trigger>
	</slot>
	<Select.Content
		class="border-primary-600 bg-primary-800 flex max-h-96 flex-col gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
		{...dropTransition}
		{avoidCollisions}
	>
		{#each items as item}
			<slot name="item" {item}>
				<Select.Item
					value={item}
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-200 flex cursor-default items-center rounded-md px-3 py-1 text-left"
				>
					{getLabel(item)}

					<Select.ItemIndicator class="ml-auto">
						<Icon icon="mdi:check" class="text-accent-400 text-lg" />
					</Select.ItemIndicator>
				</Select.Item>
			</slot>
		{/each}
	</Select.Content>
</Select.Root>
