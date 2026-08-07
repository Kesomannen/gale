<script lang="ts">
	import { DragDropProvider, DragOverlay } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import { PointerActivationConstraints, PointerSensor } from '@dnd-kit/dom';
	import type { Folder, ListItem, Mod, ModContextItem } from '$lib/types';
	import VirtualList from '../ui/VirtualList.svelte';
	import ProfileModCard from './ProfileModCard.svelte';
	import ProfileModListFolderItem from './ProfileModListFolderItem.svelte';
	import ProfileModListItem from './ProfileModListItem.svelte';
	import ProfileModCardWithContext from './ProfileModCardWithContext.svelte';

	type Props = {
		items: ListItem[];
		reorderable?: boolean;
		locked?: boolean;
		selectedMod: Mod | null;
		onModToggle: (mod: Mod, newState: boolean) => void;
		modContextItems: ModContextItem[];
	};

	let {
		items = $bindable(),
		reorderable,
		locked = false,
		selectedMod = $bindable(),
		onModToggle,
		modContextItems
	}: Props = $props();

	type DragIdentifier = string | number;
	type DropState = 'before' | 'after' | 'folder';

	let draggingId = $state<DragIdentifier | null>(null);
	let draggingIndex = $state<number | null>(null);

	const draggingItem = $derived(draggingIndex === null ? null : itemByIndex(items, draggingIndex));

	let targetIndex = $state<number | null>(null);
	let dropState = $state<DropState | null>(null);

	function itemId(item: ListItem): string {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
	}

	function itemByIndex(items: ListItem[], index: number): ListItem | null {
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

	function isTopLevelIndex(items: ListItem[], index: number): boolean {
		const { folderIndex } = unwrapIndex(items, index);
		return folderIndex === null;
	}

	function unwrapIndex(
		items: ListItem[],
		index: number
	): { folderIndex: number | null; innerIndex: number } {
		if (index < items.length) {
			return { folderIndex: null, innerIndex: index };
		}

		index -= items.length;
		const folderIndex = Math.floor(index / items.length);
		const innerIndex = index % items.length;
		return { folderIndex, innerIndex };
	}

	function findItemWithId(id: DragIdentifier): ListItem | undefined {
		for (const item of items) {
			if (itemId(item) === id) {
				return item;
			}

			if (item.type !== 'folder') {
				continue;
			}

			const found = item.folder.mods.find((mod) => mod.uuid === id);
			if (found) {
				return { type: 'mod', mod: found };
			}
		}
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

	function insertItem(items: ListItem[], item: ListItem, index: number) {
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

	function removeItem(items: ListItem[], index: number): ListItem | null {
		const { folderIndex, innerIndex } = unwrapIndex(items, index);

		if (folderIndex === null) {
			return items.splice(innerIndex, 1)[0] ?? null;
		}

		const folderItem = items[folderIndex];
		if (!folderItem || folderItem.type !== 'folder') {
			return null;
		}

		const removed = folderItem.folder.mods.splice(innerIndex, 1)[0] ?? null;
		return removed ? { type: 'mod', mod: removed } : null;
	}

	function clearEmptyFolders(items: ListItem[]) {
		let i = 0;
		while (i < items.length) {
			const item = items[i];
			if (item.type === 'folder' && item.folder.mods.length === 0) {
				items.splice(i, 1);
			} else {
				i++;
			}
		}
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
					name: 'New Folder',
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

			// onmove?.(dragged, fromIndex, toIndex);
		}

		// we wait to clear empty folders until we are done to not shift any indices
		clearEmptyFolders(newItems);
		items = newItems;

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
					{reorderable}
					index={outerIndex}
					ghost={draggingId === item.folder.id}
					dropState={getDropState(outerIndex)}
					bind:folder={item.folder}
				>
					{#snippet mod({ mod, index: innerIndex })}
						{@render modItem((outerIndex + 1) * items.length + innerIndex, mod)}
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
	<ProfileModListItem
		{index}
		{reorderable}
		id={mod.uuid}
		ghost={draggingId === mod.uuid}
		dropState={getDropState(index)}
	>
		<ProfileModCardWithContext
			{mod}
			{locked}
			{index}
			contextItems={modContextItems}
			class="mr-2"
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
	</ProfileModListItem>
{/snippet}
