<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import { iconSrc } from '$lib/util';
	import type { MouseEventHandler } from 'svelte/elements';
	import { ContextMenu } from 'bits-ui';
	import { activeContextMenu } from '$lib/context';
	import ModContextMenuContent from './ModContextMenuContent.svelte';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		locked: boolean;
		contextItems: ModContextItem[];
		onclick?: MouseEventHandler<HTMLButtonElement>;
		oninstall?: () => void;
	};

	let { mod, isSelected, locked, contextItems, onclick, oninstall }: Props = $props();

	let contextMenuOpen: boolean = $state(false);

	let descriptionClasses = $derived(
		isSelected ? 'text-primary-300' : 'text-primary-400 group-hover:text-primary-300'
	);

	$effect(() => {
		if ($activeContextMenu !== null && $activeContextMenu !== mod.uuid) {
			contextMenuOpen = false;
		}
	});
</script>

<ContextMenu.Root
	bind:open={contextMenuOpen}
	onOpenChange={(open) => {
		if (open) {
			$activeContextMenu = mod.uuid;
		} else {
			$activeContextMenu = null;
		}
	}}
>
	<ContextMenu.Trigger class="contents">
		<button
			class="group flex w-full rounded-lg border p-2 {isSelected
				? 'border-primary-500 bg-primary-700'
				: 'hover:bg-primary-700 border-transparent'}"
			{onclick}
		>
			<img src={iconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
			<div class="shrink grow overflow-hidden pl-3 text-left">
				<div class="flex items-center gap-1 overflow-hidden">
					<div class="shrink truncate pr-1 font-medium text-white">
						{mod.name.replace(/_/g, ' ')}
					</div>
					{#if mod.isPinned}
						<Icon class="text-primary-400 shrink-0" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-red-500" icon="mdi:error" />
					{/if}
					{#if mod.isInstalled}
						<Icon class="text-accent-500 shrink-0" icon="mdi:check-circle" />
					{/if}
				</div>

				{#if mod.description !== null}
					<div class="truncate {descriptionClasses}">
						{mod.description}
					</div>
				{/if}
			</div>

			{#if !mod.isInstalled && !locked}
				<!-- svelte-ignore node_invalid_placement_ssr -->
				<!-- we're not using ssr -->
				<button
					class="bg-accent-600 hover:bg-accent-500 mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline"
					onclick={(evt) => {
						evt.stopPropagation();
						oninstall?.();
					}}
				>
					<Icon icon="mdi:download" />
				</button>
			{/if}
		</button>
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>
