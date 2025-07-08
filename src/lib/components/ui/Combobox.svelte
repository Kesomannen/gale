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
			'group bg-primary-900 hover:border-primary-500 focus-within:border-primary-500 group flex items-center gap-2 overflow-hidden rounded-lg border border-transparent pr-2 disabled:cursor-not-allowed'
		]}
	>
		<Combobox.Input
			{placeholder}
			oninput={handleInput}
			clearOnDeselect
			class="placeholder-primary-400 text-primary-300 h-full w-full py-1.5 pl-3 focus:outline-0"
		/>
		<Combobox.Trigger><DropdownArrow class="text-primary-400" {open} /></Combobox.Trigger>
	</div>
	<Combobox.Portal>
		<Combobox.Content forceMount>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-primary-600 bg-primary-800 flex max-h-96 w-[var(--bits-combobox-anchor-width)] gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							{#each filteredItems as item, i (i + item.value)}
								<Combobox.Item
									{...item}
									class="hover:bg-primary-700 hover:text-primary-200 group flex w-full cursor-default items-center rounded-md px-3 py-1"
								>
									{#snippet children({ selected, highlighted })}
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
								</Combobox.Item>
							{:else}
								<span class="w-full text-center text-primary-400 py-1"> No results found </span>
							{/each}
						</div>
					{/if}
				</div>
			{/snippet}
		</Combobox.Content>
	</Combobox.Portal>
</Combobox.Root>
