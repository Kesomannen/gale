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
			'group enabled:hover:border-primary-400 dark:bg-primary-900 dark:enabled:hover:border-primary-500 bg-primary-100 flex items-center gap-2 overflow-hidden rounded-lg border border-transparent py-1 pr-2 pl-3'
		]}
	>
		{#if icon}
			<Icon class="text-primary-500 dark:text-primary-400 shrink-0 text-lg" {icon} />
		{/if}

		{#if label && typeof label !== 'string'}
			{@render label({ defaultLabel: selectedLabel ?? null })}
		{:else}
			<div
				class={[
					label || selectedLabel
						? 'text-primary-700 dark:text-primary-300'
						: 'text-primary-500 dark:text-primary-400',
					'group-disabled:text-primary-500 dark:group-disabled:text-primary-400 shrink grow truncate text-left'
				]}
			>
				{label ?? selectedLabel ?? placeholder}
			</div>
		{/if}

		<DropdownArrow
			{open}
			class="text-primary-500 group-disabled:text-primary-500 dark:text-primary-400 dark:group-disabled:text-primary-500 ml-auto"
		/>
	</Select.Trigger>
	<Select.Portal>
		<Select.Content forceMount {avoidCollisions}>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-primary-300 dark:border-primary-600 dark:bg-primary-800 flex max-h-96 w-(--bits-select-anchor-width) gap-0.5 overflow-y-auto rounded-lg border bg-white p-1 shadow-xl"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							<Select.Viewport>
								{#each items as item, i (i + item.value)}
									<Select.Item
										{...item}
										class="hover:text-primary-800 group dark:hover:bg-primary-700 dark:hover:text-primary-200 hover:bg-primary-200 flex w-full cursor-default items-center rounded-md px-3 py-1"
									>
										{#snippet children({ selected })}
											{#if itemSnippet}
												{@render itemSnippet({ ...item, selected })}
											{/if}

											<span
												class={[
													selected
														? 'text-primary-700 dark:text-primary-300'
														: 'text-primary-500 group-hover:text-primary-700 dark:text-primary-400 dark:group-hover:text-primary-300'
												]}>{item.label}</span
											>

											{#if selected}
												<Icon
													icon="mdi:check"
													class="text-accent-600 dark:text-accent-400 ml-auto shrink-0 text-lg"
												/>
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
