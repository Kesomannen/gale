<script lang="ts">
	import { dropIn, dropOut } from '$lib/transitions';
	import Icon from '@iconify/svelte';
	import { Combobox, type WithoutChildrenOrChild, mergeProps } from 'bits-ui';
	import type { ClassValue } from 'clsx';
	import { fade, fly } from 'svelte/transition';
	import DropdownArrow from './DropdownArrow.svelte';

	type Props = Combobox.RootProps & {
		triggerClass?: ClassValue;
		placeholder?: string;
	};

	let {
		items = [],
		value = $bindable(),
		open = $bindable(false),
		triggerClass,
		placeholder,
		type,
		...restProps
	}: Props = $props();

	let searchValue = $state('');

	const filteredItems = $derived.by(() => {
		if (searchValue === '') return items;
		return items.filter((item) => item.label.toLowerCase().includes(searchValue.toLowerCase()));
	});

	function handleInput(evt: Event & { currentTarget: HTMLInputElement }) {
		searchValue = evt.currentTarget.value;
	}

	function handleOpenChange(newOpen: boolean) {
		if (!newOpen) searchValue = '';
	}

	const mergedRootProps = $derived(mergeProps(restProps, { onOpenChange: handleOpenChange }));
</script>

<Combobox.Root {type} {items} bind:value={value as never} bind:open {...mergedRootProps}>
	<div
		class={[
			triggerClass,
			'group hover:border-primary-400 focus-within:border-primary-400 group dark:bg-primary-900 dark:hover:border-primary-500 dark:focus-within:border-primary-500 bg-primary-100 flex items-center gap-2 overflow-hidden rounded-lg border border-transparent pr-2 disabled:cursor-not-allowed'
		]}
	>
		<Combobox.Input
			{placeholder}
			oninput={handleInput}
			clearOnDeselect
			class="placeholder-primary-500 text-primary-700 dark:placeholder-primary-400 dark:text-primary-300 h-full w-full py-1.5 pl-3 focus:outline-0"
		/>
		<Combobox.Trigger
			><DropdownArrow class="text-primary-500 dark:text-primary-400" {open} /></Combobox.Trigger
		>
	</div>
	<Combobox.Portal>
		<Combobox.Content forceMount>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-primary-300 dark:border-primary-600 dark:bg-primary-800 flex max-h-96 w-[var(--bits-combobox-anchor-width)] gap-0.5 overflow-y-auto rounded-lg border bg-white p-1 shadow-xl"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							{#each filteredItems as item, i (i + item.value)}
								<Combobox.Item
									{...item}
									class="hover:text-primary-800 group dark:hover:bg-primary-700 dark:hover:text-primary-200 hover:bg-primary-200 flex w-full cursor-default items-center rounded-md px-3 py-1"
								>
									{#snippet children({ selected, highlighted })}
										<span
											class={[
												selected || highlighted
													? 'text-primary-700 dark:text-primary-300'
													: 'text-primary-500 group-hover:text-primary-700 dark:text-primary-400 dark:group-hover:text-primary-300'
											]}>{item.label}</span
										>

										{#if selected}
											<Icon
												icon="mdi:check"
												class="text-accent-600 dark:text-accent-400 ml-auto text-lg"
											/>
										{/if}
									{/snippet}
								</Combobox.Item>
							{:else}
								<span class="w-full text-center text-primary-500 py-1 dark:text-primary-400">
									No results found
								</span>
							{/each}
						</div>
					{/if}
				</div>
			{/snippet}
		</Combobox.Content>
	</Combobox.Portal>
</Combobox.Root>
