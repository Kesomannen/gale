<script lang="ts">
	import { emptyOrUndefined } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';

	import type { Snippet } from 'svelte';

	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: { value: string; label: string; disabled?: boolean }[];
		triggerClass?: string;
		icon?: string;
		avoidCollisions?: boolean;
	} & (
			| {
					label?: never;
			  }
			| {
					placeholder?: never;
					label: Snippet;
			  }
			| {
					placeholder?: never;
					label: string;
			  }
		);

	let {
		open = $bindable(false),
		value = $bindable(),
		triggerClass,
		items,
		placeholder,
		icon,
		avoidCollisions,
		label,
		...restProps
	}: Props = $props();

	const selectedLabel = $derived(
		restProps.type === 'single'
			? items.find((item) => item.value === value)?.label
			: emptyOrUndefined(
					items
						.filter((item) => value?.includes(item.value))
						.map((item) => item.label)
						.join(', ')
				)
	);
</script>

<Select.Root bind:value={value as never} bind:open {...restProps}>
	<Select.Trigger
		class={[
			triggerClass,
			'group bg-primary-900 enabled:hover:border-primary-500 flex items-center gap-2 overflow-hidden rounded-lg border border-transparent py-1 pr-2 pl-3 disabled:cursor-not-allowed'
		]}
	>
		{#if icon}
			<Icon class="text-primary-400 shrink-0 text-lg" {icon} />
		{/if}

		{#if label && typeof label !== 'string'}
			{@render label()}
		{:else}
			<div
				class={[
					label || selectedLabel ? 'text-primary-300' : 'text-primary-400',
					' group-disabled:text-primary-400 shrink grow truncate text-left'
				]}
			>
				{label ?? selectedLabel ?? placeholder}
			</div>
		{/if}

		<!-- can't use a class array here because of bug in iconify Icon -->
		<Icon
			class="text-primary-400 group-disabled:text-primary-500 ml-auto shrink-0 origin-center transform text-lg transition-transform duration-100 ease-out {open
				? 'rotate-180'
				: 'rotate-0'}"
			icon="mdi:chevron-down"
		/>
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			{avoidCollisions}
			class="border-primary-600 bg-primary-800 flex max-h-96 w-full gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
		>
			<Select.Viewport>
				{#each items as item, i (i + item.value)}
					<Select.Item
						{...item}
						class="hover:bg-primary-700 hover:text-primary-200 flex w-full cursor-default items-center rounded-md px-3 py-1"
					>
						{#snippet children({ selected })}
							<span class="text-primary-400">{item.label}</span>

							{#if selected}
								<Icon icon="mdi:check" class="text-accent-400 ml-auto text-lg" />
							{/if}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Viewport>
		</Select.Content>
	</Select.Portal>
</Select.Root>
