<script lang="ts">
	import { DragDropProvider, DragOverlay } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import { PointerActivationConstraints, PointerSensor } from '@dnd-kit/dom';
	import type { Folder, ProfileListItem, Mod, ModContextItem } from '$lib/types';
	import VirtualList from '../ui/VirtualList.svelte';
	import ProfileModCard from './ProfileModCard.svelte';
	import ProfileModListFolderItem from './ProfileModListFolderItem.svelte';
	import ProfileModCardWithContext from './ProfileModCardWithContext.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		items: ProfileListItem[];
		reorderable?: boolean;
		locked?: boolean;
		selectedMod: Mod | null;
		onModToggle: (mod: Mod, newState: boolean) => void;
		onFolderToggle?: (folderId: string, newState: boolean) => void;
		modContextItems: ModContextItem<Mod>[];
		folderContextItems: ModContextItem<Folder>[];
		onLayoutChange?: (items: ProfileListItem[]) => void;
	};

	let {
		items = $bindable(),
		reorderable,
		locked = false,
		selectedMod = $bindable(),
		onModToggle,
		onFolderToggle,
		modContextItems,
		folderContextItems,
		onLayoutChange
	}: Props = $props();

	type DragIdentifier = string | number;
	type DropState = 'before' | 'after' | 'folder';

	let draggingId = $state<DragIdentifier | null>(null);
	let draggingIndex = $state<number | null>(null);

	const draggingItem = $derived(draggingIndex === null ? null : itemByIndex(items, draggingIndex));

	let targetIndex = $state<number | null>(null);
	let dropState = $state<DropState | null>(null);

	function itemId(item: ProfileListItem): string {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
	}

	function itemByIndex(items: ProfileListItem[], index: number): ProfileListItem | null {
		if (index < 0) {
			return null;
		}

		const { folderIndex, innerIndex } = unwrapIndex(items, index);

		if (folderIndex == null) {
			// top-level item
			return items[index] ?? null;
		}

		// mod in a folder
		const folderItem = items[folderIndex];
		if (!folderItem || folderItem.type !== 'folder') {
			return null;
		}

		const mod = folderItem.folder.mods[innerIndex];
		if (!mod) {
			return null;
		}

		return { type: 'mod', mod };
	}

	function isTopLevelIndex(items: ProfileListItem[], index: number): boolean {
		const { folderIndex } = unwrapIndex(items, index);
		return folderIndex === null;
	}

	function unwrapIndex(
		items: ProfileListItem[],
		index: number
	): { folderIndex: number | null; innerIndex: number } {
		if (index <= items.length) {
			return { folderIndex: null, innerIndex: index };
		}

		index -= items.length + 1;
		const folderIndex = Math.floor(index / items.length);
		const innerIndex = index % items.length;
		return { folderIndex, innerIndex };
	}

	function resetDragState() {
		draggingId = null;
		draggingIndex = null;

		targetIndex = null;
		dropState = null;
	}

	function onDragStart(event: any) {
		const { source } = event.operation;
		resetDragState();

		if (!isSortable(source)) {
			return;
		}

		draggingId = source.id;
		draggingIndex = source.index;
		// draggingIndex = items.findIndex((item) => itemId(item) === source.id);
	}

	function onDragMove(event: any) {
		const { source, target } = event.operation;

		if (!isSortable(source) || !isSortable(target) || source.id === target.id || !draggingItem) {
			return;
		}

		const targetElement = target.element as HTMLElement | undefined;
		const pointerY = event.to?.y;
		if (!targetElement || pointerY === undefined) {
			return;
		}

		targetIndex = target.index;

		const rect = targetElement.getBoundingClientRect();
		const percentageY = (pointerY - rect.top) / rect.height;
		const isInCenter = percentageY > 0.35 && percentageY < 0.65;

		const { folderIndex: targetFolderIndex } = unwrapIndex(items, targetIndex);

		if (draggingItem.type === 'mod' && targetFolderIndex === null && isInCenter) {
			// dragging a mod over the middle of another top-level item
			// if the item is a folder, we will drop it into the folder, otherwise we create a new one
			dropState = 'folder';
			return;
		}

		dropState = percentageY < 0.5 ? 'before' : 'after';
	}

	function insertItem(items: ProfileListItem[], item: ProfileListItem, index: number) {
		const { folderIndex, innerIndex } = unwrapIndex(items, index);

		if (folderIndex === null) {
			items.splice(innerIndex, 0, item);
			return;
		}

		if (item.type !== 'mod') {
			// nested folders are not allowed
			return;
		}

		const folderItem = items[folderIndex];
		if (!folderItem || folderItem.type !== 'folder') {
			return;
		}

		folderItem.folder.mods.splice(innerIndex, 0, item.mod);
	}

	function removeItem(items: ProfileListItem[], index: number): ProfileListItem | null {
		const { folderIndex, innerIndex } = unwrapIndex(items, index);

		if (folderIndex === null) {
			// top-level item
			return items.splice(innerIndex, 1)[0] ?? null;
		}

		// mod in a folder; first find the folder item
		const folderItem = items[folderIndex];
		if (!folderItem || folderItem.type !== 'folder') {
			return null;
		}

		const removed = folderItem.folder.mods.splice(innerIndex, 1)[0] ?? null;
		return removed ? { type: 'mod', mod: removed } : null;
	}

	function onDragEnd(event: any) {
		if (event.canceled || draggingIndex === null || targetIndex === null) {
			resetDragState();
			return;
		}

		const fromIndex = draggingIndex;
		let toIndex = getFinalDropIndex(fromIndex, targetIndex, dropState);

		if (fromIndex === toIndex) {
			resetDragState();
			return;
		}

		const newItems = [...items];

		if (dropState === 'folder') {
			// if hovering a mod, create a new folder
			// if hovering a folder, move the dragged mod into the folder

			// the target is guaranteed to be a top-level item as nested folders are not allowed
			const target = newItems[toIndex];

			// the draggedItem can be nested in a folder
			const dragged = removeItem(newItems, fromIndex);
			if (!dragged || dragged.type !== 'mod') {
				resetDragState();
				return;
			}

			if (target.type === 'folder') {
				// move the dragged mod into the target folder
				target.folder.mods.push(dragged.mod);
			} else {
				// create a new folder with the dragged mod and the target mod
				// remove the target item to replace it with a folder
				if (isTopLevelIndex(newItems, fromIndex) && fromIndex < toIndex) {
					// if the dragged item is above the target,
					// we need to adjust the index since we just removed the dragged item from the list
					toIndex--;
				}
				newItems.splice(toIndex, 1);

				const folder: Folder = {
					id: crypto.randomUUID(),
					name: m.page_folderNew(),
					mods: [dragged.mod, target.mod],
					isExpanded: false
				};

				newItems.splice(toIndex, 0, { type: 'folder', folder });
			}
		} else {
			// reorder the item by removing it from the list and inserting it at the new index
			const dragged = removeItem(newItems, fromIndex);
			if (!dragged) {
				resetDragState();
				return;
			}

			insertItem(newItems, dragged, toIndex);
		}

		items = newItems;

		onLayoutChange?.(items);
		resetDragState();
	}

	function getFinalDropIndex(
		fromIndex: number,
		targetIndex: number,
		state: DropState | null
	): number {
		if (state === 'folder') {
			return targetIndex;
		}
		if (state === 'before') {
			return fromIndex < targetIndex ? targetIndex - 1 : targetIndex;
		}
		return fromIndex < targetIndex ? targetIndex : targetIndex + 1;
	}

	function getDropState(index: number): DropState | null {
		if (targetIndex === null || draggingIndex === null || targetIndex === draggingIndex) {
			return null;
		}

		if (targetIndex === index) {
			return dropState;
		}

		return null;
	}
</script>

<DragDropProvider
	{onDragStart}
	{onDragMove}
	{onDragEnd}
	sensors={(defaults) => [
		...defaults.filter((sensor) => sensor !== PointerSensor),
		PointerSensor.configure({
			activationConstraints: [new PointerActivationConstraints.Distance({ value: 6 })]
		})
	]}
>
	<VirtualList
		{items}
		rowId={(item) => itemId(item)}
		pinnedRowIds={draggingId !== null ? [draggingId] : []}
	>
		{#snippet children({ item, index: outerIndex })}
			{#if item.type === 'folder'}
				<ProfileModListFolderItem
					bind:folder={item.folder}
					{reorderable}
					{locked}
					index={outerIndex}
					ghost={draggingId === item.folder.id}
					dropState={getDropState(outerIndex)}
					contextItems={folderContextItems}
					ontoggle={(newState) => onFolderToggle?.(item.folder.id, newState)}
				>
					{#snippet mod({ mod, index: innerIndex })}
						{@render modItem((outerIndex + 1) * items.length + innerIndex + 1, mod)}
					{/snippet}
				</ProfileModListFolderItem>
			{:else}
				{@render modItem(outerIndex, item.mod)}
			{/if}
		{/snippet}
	</VirtualList>

	<DragOverlay dropAnimation={{ duration: 150, easing: 'cubic-bezier(0.5, 1, 0.89, 1)' }}>
		{#if draggingItem}
			{#if draggingItem.type === 'folder'}
				<div>folder</div>
			{:else}
				<ProfileModCard mod={draggingItem.mod} />
			{/if}
		{/if}
	</DragOverlay>
</DragDropProvider>

{#snippet modItem(index: number, mod: Mod)}
	<ProfileModCardWithContext
		{mod}
		{locked}
		{index}
		{reorderable}
		dropState={getDropState(index)}
		contextItems={modContextItems}
		ghost={draggingId === mod.uuid}
		selected={selectedMod?.uuid === mod.uuid}
		ontoggle={(newState) => onModToggle(mod, newState)}
		onclick={() => {
			if (selectedMod?.uuid === mod.uuid) {
				selectedMod = null;
			} else {
				selectedMod = mod;
			}
		}}
	/>
{/snippet}
