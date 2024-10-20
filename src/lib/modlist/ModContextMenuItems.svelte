<script lang="ts">
	import type { Mod, ModContextItem } from '$lib/models';
	import { ContextMenu, DropdownMenu } from 'bits-ui';
	import Icon from '@iconify/svelte';

	export let mod: Mod;
	export let contextItems: ModContextItem[];
	export let type: 'details' | 'context';
</script>

{#each contextItems as { icon, label, onclick, showFor }}
	{#if showFor === undefined || showFor(mod)}
		{#if type === 'details'}
			<DropdownMenu.Item
				class="menu-item text-slate-300 hover:bg-gray-600 hover:text-slate-100"
				on:click={() => onclick(mod)}
			>
				<Icon class="mr-1.5 text-lg" {icon} />
				{label}
			</DropdownMenu.Item>
		{:else}
			<ContextMenu.Item
				class="menu-item text-slate-400 hover:bg-gray-700 hover:text-slate-200"
				on:click={() => onclick(mod)}
			>
				<Icon class="mr-1.5 text-lg" {icon} />
				{label}
			</ContextMenu.Item>
		{/if}
	{/if}
{/each}

<style lang="postcss">
	:global(.menu-item) {
		@apply flex cursor-default items-center truncate rounded py-1 pl-3 pr-5 text-left;
	}
</style>
