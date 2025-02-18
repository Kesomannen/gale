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
			class="group flex items-center gap-2 overflow-hidden rounded-lg border border-transparent bg-slate-900 py-1 pr-2 pl-3 enabled:hover:border-slate-500 disabled:cursor-not-allowed {className}"
		>
			{#if icon}
				<Icon class="shrink-0 text-lg text-slate-400" {icon} />
			{/if}

			<div
				class="shrink grow truncate text-left text-slate-300 group-disabled:text-slate-400"
				class:text-slate-300={overrideLabel || label}
				class:text-slate-400={!overrideLabel && !label}
			>
				{overrideLabel ?? label ?? placeholder}
			</div>

			<Icon
				class="shrink-0 origin-center transform text-lg text-slate-400 transition-all duration-100 ease-out group-disabled:text-slate-500 {open
					? 'rotate-180'
					: 'rotate-0'}"
				icon="mdi:chevron-down"
			/>
		</Select.Trigger>
	</slot>
	<Select.Content
		class="flex max-h-96 flex-col gap-0.5 overflow-y-auto rounded-lg border border-slate-600 bg-slate-800 p-1 shadow-xl"
		{...dropTransition}
		{avoidCollisions}
	>
		{#each items as item}
			<slot name="item" {item}>
				<Select.Item
					value={item}
					class="flex cursor-default items-center rounded-md px-3 py-1 text-left text-slate-400 hover:bg-slate-700 hover:text-slate-200"
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
