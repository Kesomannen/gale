<script lang="ts" context="module">
	import { writable } from 'svelte/store';

	export let activeContextMenu = writable<string | null>(null);
</script>

<script lang="ts">
	import type { Mod, ModContextItem } from '../models';
	import Icon from '@iconify/svelte';
	import { iconSrc, isOutdated } from '$lib/util';
	import { Switch, ContextMenu } from 'bits-ui';
	import { createEventDispatcher } from 'svelte';
	import ModContextMenuItems from './ModContextMenuItems.svelte';
	import { dropTransition } from '$lib/transitions';

	export let mod: Mod;
	export let index: number;
	export let isSelected: boolean;
	export let contextItems: ModContextItem[];

	export let reorderable: boolean;
	export let locked: boolean;

	const dispatch = createEventDispatcher<{
		toggle: boolean;
	}>();

	let contextMenuOpen: boolean;

	$: descriptionClasses =
		mod.enabled === false
			? 'text-primary-500 line-through'
			: isSelected
				? 'text-primary-300'
				: 'text-primary-400 group-hover:text-primary-300';

	$: if ($activeContextMenu !== null && $activeContextMenu !== mod.uuid) {
		contextMenuOpen = false;
	}
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
			class="group flex w-full items-center rounded-lg border p-2 {isSelected
				? 'border-primary-500 bg-primary-700'
				: 'hover:bg-primary-700 border-transparent'}"
			data-uuid={mod.uuid}
			data-index={index}
			draggable={reorderable}
			on:click
			on:dragstart
			on:dragover
			on:dragend
		>
			<img src={iconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
			<div class="shrink grow overflow-hidden pr-2 pl-3 text-left">
				<div class="flex items-center gap-1 overflow-hidden">
					<div
						class="shrink truncate font-medium {mod.enabled === false
							? 'text-primary-300 line-through'
							: 'text-white'}"
					>
						{mod.name.replace(/_/g, ' ')}
					</div>
					<div class="px-1 {descriptionClasses}">
						{mod.version ?? '?.?.?'}
					</div>
					{#if mod.isPinned}
						<Icon class="text-primary-400 shrink-0" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-red-500" icon="mdi:error" />
					{/if}
					{#if isOutdated(mod)}
						<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
					{/if}
				</div>

				{#if mod.description !== null}
					<div class="truncate {descriptionClasses}">
						{mod.description}
					</div>
				{/if}
			</div>

			{#if reorderable && !locked}
				<Icon
					icon="material-symbols:drag-indicator"
					class="text-primary-400 mr-4 shrink-0 cursor-move text-2xl"
				/>
			{/if}

			<!-- make sure click events don't propagate and cause the mod to be selected -->
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<!-- svelte-ignore a11y-no-static-element-interactions -->
			<div class="contents" on:click={(evt) => evt.stopPropagation()}>
				<Switch.Root
					disabled={locked}
					checked={mod.enabled ?? true}
					onCheckedChange={(newState) => dispatch('toggle', newState)}
					class="group data-[state=checked]:bg-accent-700 data-[state=checked]:hover:bg-accent-600 bg-primary-600 hover:bg-primary-500 mr-1 flex h-6 w-12 shrink-0 rounded-full px-1 py-1"
				>
					<Switch.Thumb
						class="data-[state=checked]:bg-accent-200 bg-primary-300 pointer-events-none h-full w-4 rounded-full transition-transform duration-75 ease-out data-[state=checked]:translate-x-6"
					/>
				</Switch.Root>
			</div>
		</button>
	</ContextMenu.Trigger>
	<ContextMenu.Content
		class="border-primary-600 bg-primary-800 flex flex-col gap-0.5 rounded-lg border p-1 shadow-lg"
		{...dropTransition}
	>
		<ModContextMenuItems {mod} {contextItems} {locked} type="context" />
	</ContextMenu.Content>
</ContextMenu.Root>
