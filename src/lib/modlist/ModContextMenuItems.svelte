<script lang="ts">
	import ModContextMenuItems from './ModContextMenuItems.svelte';
	import type { Mod, ModContextItem } from '$lib/types';
	import { ContextMenu, DropdownMenu } from 'bits-ui';
	import Icon from '@iconify/svelte';

	type Props = {
		mod: Mod;
		contextItems: ModContextItem[];
		type: 'details' | 'context';
		locked: boolean;
	};

	let { mod, contextItems, type, locked }: Props = $props();
</script>

{#each contextItems as { icon, label, onclick, showFor, children }}
	{#if showFor === undefined || showFor(mod, locked)}
		{#if type === 'details'}
			{#if children === undefined}
				<DropdownMenu.Item class="menu-item dropdown-menu-item pr-6" onclick={() => onclick(mod)}>
					{#if icon !== undefined}
						<Icon class="mr-1.5 text-lg" {icon} />
					{/if}

					{label}
				</DropdownMenu.Item>
			{:else}
				<DropdownMenu.Sub>
					<DropdownMenu.SubTrigger
						class="menu-item dropdown-menu-item pr-2"
						onclick={() => onclick(mod)}
					>
						{#if icon !== undefined}
							<Icon class="mr-1.5 text-lg" {icon} />
						{/if}

						{label}

						<Icon class="ml-auto text-lg" icon="mdi:chevron-right" />
					</DropdownMenu.SubTrigger>
					<DropdownMenu.SubContent
						class="sub-menu light-scrollbar border-primary-500 bg-primary-700"
					>
						<ModContextMenuItems {locked} {mod} {type} contextItems={children(mod)} />
					</DropdownMenu.SubContent>
				</DropdownMenu.Sub>
			{/if}
		{:else if children === undefined}
			<ContextMenu.Item class="menu-item context-menu-item pr-6" onclick={() => onclick(mod)}>
				{#if icon !== undefined}
					<Icon class="mr-1.5 text-lg" {icon} />
				{/if}

				{label}
			</ContextMenu.Item>
		{:else}
			<ContextMenu.Sub>
				<ContextMenu.SubTrigger
					class="menu-item context-menu-item pr-2"
					onclick={() => onclick(mod)}
				>
					{#if icon !== undefined}
						<Icon class="mr-1.5 text-lg" {icon} />
					{/if}

					{label}

					<Icon class="ml-auto text-lg" icon="mdi:chevron-right" />
				</ContextMenu.SubTrigger>
				<ContextMenu.SubContent class="sub-menu border-primary-600 bg-primary-800">
					<ModContextMenuItems {locked} {mod} {type} contextItems={children(mod)} />
				</ContextMenu.SubContent>
			</ContextMenu.Sub>
		{/if}
	{/if}
{/each}
