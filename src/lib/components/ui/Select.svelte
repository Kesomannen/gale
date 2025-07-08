<script lang="ts">
	import { dropIn, dropOut } from '$lib/transitions';
	import { emptyOrUndefined } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';

	import type { Snippet } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import DropdownArrow from './DropdownArrow.svelte';

	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: { value: string; label: string; disabled?: boolean }[];
		triggerClass?: string;
		icon?: string;
		avoidCollisions?: boolean;
		item?: Snippet<[{ label: string; value: string; selected: boolean }]>;
	} & (
			| {
					label?: never;
			  }
			| {
					placeholder?: never;
					label: Snippet<[{ defaultLabel: string | null }]>;
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
		item: itemSnippet,
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
			{@render label({ defaultLabel: selectedLabel ?? null })}
		{:else}
			<div
				class={[
					label || selectedLabel ? 'text-primary-300' : 'text-primary-400',
					'group-disabled:text-primary-400 shrink grow truncate text-left'
				]}
			>
				{label ?? selectedLabel ?? placeholder}
			</div>
		{/if}

		<DropdownArrow {open} class="text-primary-400 group-disabled:text-primary-500 ml-auto" />
	</Select.Trigger>
	<Select.Portal>
		<Select.Content forceMount {avoidCollisions}>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-primary-600 bg-primary-800 flex max-h-96 w-[var(--bits-select-anchor-width)] gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							<Select.Viewport>
								{#each items as item, i (i + item.value)}
									<Select.Item
										{...item}
										class="hover:bg-primary-700 hover:text-primary-200 group flex w-full cursor-default items-center rounded-md px-3 py-1"
									>
										{#snippet children({ selected, highlighted })}
											{#if itemSnippet}
												{@render itemSnippet({ ...item, selected })}
											{/if}

											<span
												class={[
													selected || highlighted
														? 'text-primary-300'
														: 'text-primary-400 group-hover:text-primary-300'
												]}>{item.label}</span
											>

											{#if selected}
												<Icon icon="mdi:check" class="text-accent-400 ml-auto text-lg" />
											{/if}
										{/snippet}
									</Select.Item>
								{/each}
							</Select.Viewport>
						</div>
					{/if}
				</div>
			{/snippet}
		</Select.Content>
	</Select.Portal>
</Select.Root>
