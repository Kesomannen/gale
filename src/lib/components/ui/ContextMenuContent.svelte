<script lang="ts">
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import type { ContextItem, Mod } from '$lib/types';
	import { ContextMenu, DropdownMenu } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { fade, fly } from 'svelte/transition';
	import { dropIn, dropOut } from '$lib/transitions';
	import type { ClassValue } from 'clsx';

	type Props = {
		items: ContextItem[];
		type: 'dropdown' | 'context';
		style: 'dark' | 'light';
		sub?: boolean;
		class?: ClassValue;
	};

	let { items, type, style, sub = false, class: classProp }: Props = $props();

	const commonContentClass = 'flex flex-col gap-0.5 rounded-lg border p-1 shadow-xl';
	const commonItemClass =
		'flex shrink-0 cursor-default items-center truncate rounded-sm px-3 py-1 text-left';
	const submenuClass = 'max-h-80 overflow-y-auto';

	let { contentClass, itemClass } = $derived(
		{
			dark: {
				contentClass: 'border-primary-600 bg-primary-800',
				itemClass: 'text-primary-400 hover:text-primary-200 hover:bg-primary-700'
			},
			light: {
				contentClass: 'border-primary-500 bg-primary-700',
				itemClass: 'text-primary-300 hover:text-primary-100 hover:bg-primary-600'
			}
		}[style]
	);

	let { Content, Item, Sub, SubTrigger } = $derived(
		{
			dropdown: {
				Content: sub ? DropdownMenu.SubContent : DropdownMenu.Content,
				Item: DropdownMenu.Item,
				Sub: DropdownMenu.Sub,
				SubTrigger: DropdownMenu.SubTrigger
			},
			context: {
				Content: sub ? ContextMenu.SubContent : ContextMenu.Content,
				Item: ContextMenu.Item,
				Sub: ContextMenu.Sub,
				SubTrigger: ContextMenu.SubTrigger
			}
		}[type]
	);
</script>

{#if items.length > 0}
	<Content forceMount>
		{#snippet child({ wrapperProps, props, open })}
			<div {...wrapperProps}>
				{#if open}
					<div
						{...props}
						class={[commonContentClass, contentClass, sub && submenuClass, classProp]}
						in:fly={dropIn}
						out:fade={dropOut}
					>
						{#each items as { icon, label, onclick, children }}
							{#if children}
								<Sub>
									<SubTrigger class={[commonItemClass, itemClass, 'pr-2']} {onclick}>
										{#if icon}
											<Icon class="mr-1.5 text-lg" {icon} />
										{/if}

										{label}

										<Icon class="ml-auto text-lg" icon="mdi:chevron-right" />
									</SubTrigger>
									<ContextMenuContent {type} {style} sub items={children} />
								</Sub>
							{:else}
								<Item class={[commonItemClass, itemClass, 'pr-6']} {onclick}>
									{#if icon}
										<Icon class="mr-1.5 text-lg" {icon} />
									{/if}

									{label}
								</Item>
							{/if}
						{/each}
					</div>
				{/if}
			</div>
		{/snippet}
	</Content>
{/if}
