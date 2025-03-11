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
						class="sub-menu dropdown-sub-menu border-slate-500 bg-slate-700"
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
				<ContextMenu.SubContent class="sub-menu border-slate-600 bg-slate-800" {...dropTransition}>
					<svelte:self {mod} {type} contextItems={children(mod)} />
				</ContextMenu.SubContent>
			</ContextMenu.Sub>
		{/if}
	{/if}
{/each}

<style lang="postcss">
	@reference 'tailwindcss';

	:global(.menu-item) {
		@apply flex shrink-0 cursor-default items-center truncate rounded-sm py-1 pl-3 text-left;
	}

	:global(.dropdown-menu-item) {
		@apply text-slate-300 hover:bg-slate-600 hover:text-slate-100;
	}

	:global(.context-menu-item) {
		@apply text-slate-400 hover:bg-slate-700 hover:text-slate-200;
	}

	:global(.sub-menu) {
		@apply flex max-h-80 flex-col gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-lg;
	}

	:global(.dropdown-sub-menu) {
		scrollbar-color: var(--color-slate-400) var(--color-slate-700);
	}
</style>
