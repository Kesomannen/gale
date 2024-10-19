<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu, ContextMenu } from 'bits-ui';

	export let label: string;
	export let icon: string;
	export let type: 'details' | 'context';
</script>

{#if type === 'details'}
	<DropdownMenu.Item
		class="menu-item text-slate-300 hover:bg-gray-600 hover:text-slate-100"
		on:click
	>
		<Icon class="mr-1.5 text-lg" {icon} />
		{label}
	</DropdownMenu.Item>
{:else}
	<ContextMenu.Item
		class="menu-item text-slate-300 hover:bg-gray-600 hover:text-slate-100"
		on:click
	>
		<Icon class="mr-1.5 text-lg" {icon} />
		{label}
	</ContextMenu.Item>
{/if}

<style lang="postcss">
	:global(.menu-item) {
		@apply flex cursor-default items-center truncate rounded py-1 pl-3 pr-5 text-left;
	}
</style>
