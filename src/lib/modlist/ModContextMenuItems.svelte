<script lang="ts">
	import type { Mod, ModContextItem } from '$lib/models';
	import { ContextMenu, DropdownMenu } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { dropTransition } from '$lib/transitions';

	export let mod: Mod;
	export let contextItems: ModContextItem[];
	export let type: 'details' | 'context';
	export let locked: boolean;
</script>

{#each contextItems as { icon, label, onclick, showFor, children }}
	{#if showFor === undefined || showFor(mod, locked)}
		{#if type === 'details'}
			{#if children === undefined}
				<DropdownMenu.Item class="menu-item dropdown-menu-item pr-6" on:click={() => onclick(mod)}>
					{#if icon !== undefined}
						<Icon class="mr-1.5 text-lg" {icon} />
					{/if}

					{label}
				</DropdownMenu.Item>
			{:else}
				<DropdownMenu.Sub>
					<DropdownMenu.SubTrigger
						class="menu-item dropdown-menu-item pr-2"
						on:click={() => onclick(mod)}
					>
						{#if icon !== undefined}
							<Icon class="mr-1.5 text-lg" {icon} />
						{/if}

						{label}

						<Icon class="ml-auto text-lg" icon="mdi:chevron-right" />
					</DropdownMenu.SubTrigger>
					<DropdownMenu.SubContent
						class="sub-menu light-scrollbar border-primary-500 bg-primary-700"
						{...dropTransition}
					>
						<svelte:self {mod} {type} contextItems={children(mod)} />
					</DropdownMenu.SubContent>
				</DropdownMenu.Sub>
			{/if}
		{:else if children === undefined}
			<ContextMenu.Item class="menu-item context-menu-item pr-6" on:click={() => onclick(mod)}>
				{#if icon !== undefined}
					<Icon class="mr-1.5 text-lg" {icon} />
				{/if}

				{label}
			</ContextMenu.Item>
		{:else}
			<ContextMenu.Sub>
				<ContextMenu.SubTrigger
					class="menu-item context-menu-item pr-2"
					on:click={() => onclick(mod)}
				>
					{#if icon !== undefined}
						<Icon class="mr-1.5 text-lg" {icon} />
					{/if}

					{label}

					<Icon class="ml-auto text-lg" icon="mdi:chevron-right" />
				</ContextMenu.SubTrigger>
				<ContextMenu.SubContent
					class="sub-menu border-primary-600 bg-primary-800"
					{...dropTransition}
				>
					<svelte:self {mod} {type} contextItems={children(mod)} />
				</ContextMenu.SubContent>
			</ContextMenu.Sub>
		{/if}
	{/if}
{/each}
