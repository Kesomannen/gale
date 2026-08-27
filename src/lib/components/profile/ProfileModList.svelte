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
	import ProfileFolderCard from './ProfileFolderCard.svelte';

	type Props = {
		items: ProfileListItem[];
		reorderable?: boolean;
		locked?: boolean;
		selectedMod: Mod | null;
		modContextItems: ModContextItem<Mod>[];
		folderContextItems: ModContextItem<Folder>[];
		onModToggle: (mod: Mod, newState: boolean) => void;
		onFolderToggle?: (folderId: string, newState: boolean) => void;
		onLayoutChange?: (items: ProfileListItem[]) => void;
	};

	let {
		items = $bindable(),
		reorderable,
		locked = false,
		selectedMod = $bindable(),
		modContextItems,
		folderContextItems,
		onModToggle,
		onFolderToggle,
		onLayoutChange
	}: Props = $props();

	type DragIdentifier = string | number;
	type DropState = 'before' | 'after' | 'folder';

	let draggingId = $state<DragIdentifier | null>(null);
	let targetId = $state<DragIdentifier | null>(null);

	const draggingItem = $derived(draggingId === null ? null : findItem(items, draggingId));

	let dropState = $state<DropState | null>(null);

	const totalItems = $derived(
		items.reduce((count, item) => {
			if (item.type === 'folder') {
				return count + 1 + item.folder.mods.length;
			}
			return count + 1;
		}, 0)
	);

	function itemId(item: ProfileListItem): string {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
	}

	type ItemWithLocation =
		| { type: 'folder'; index: number; folder: Folder }
		| { type: 'mod'; folderIndex: number | null; innerIndex: number; mod: Mod };

	function findItem(items: ProfileListItem[], id: DragIdentifier): ItemWithLocation | null {
		for (let i = 0; i < items.length; i++) {
			const item = items[i];
			if (item.type === 'folder') {
				if (item.folder.id === id) {
					return { type: 'folder', index: i, folder: item.folder };
				}

				const modIndex = item.folder.mods.findIndex((mod) => mod.uuid === id);
				if (modIndex !== -1) {
					return {
						type: 'mod',
						folderIndex: i,
						innerIndex: modIndex,
						mod: item.folder.mods[modIndex]
					};
				}
			} else if (item.mod.uuid === id) {
				return { type: 'mod', folderIndex: null, innerIndex: i, mod: item.mod };
			}
		}

		return null;
	}

	function resetDragState() {
		draggingId = null;
		targetId = null;
		dropState = null;
	}

	function onDragStart(event: any) {
		const { source } = event.operation;
		resetDragState();

		if (!isSortable(source)) {
			return;
		}

		draggingId = source.id;
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

		targetId = target.id;

		const rect = targetElement.getBoundingClientRect();
		const percentageY = (pointerY - rect.top) / rect.height;
		const isInCenter = percentageY > 0.35 && percentageY < 0.65;

		if (draggingItem.type === 'mod' && isInCenter) {
			// dragging a mod over the middle of another top-level item
			// if the item is a folder, we will drop it into the folder, otherwise we create a new one
			dropState = 'folder';
			return;
		}

		dropState = percentageY < 0.5 ? 'before' : 'after';
	}

	function removeItem(items: ProfileListItem[], item: ItemWithLocation) {
		if (item.type === 'folder') {
			items.splice(item.index, 1);
		} else if (item.folderIndex === null) {
			// top-level mod
			items.splice(item.innerIndex, 1);
		} else {
			const folderItem = items[item.folderIndex];
			if (folderItem.type !== 'folder') {
				console.warn('Expected folder item at index', item.folderIndex);
				return;
			}
			folderItem.folder.mods.splice(item.innerIndex, 1);
		}
	}

	function insertItem(items: ProfileListItem[], item: ItemWithLocation) {
		if (item.type === 'folder') {
			items.splice(item.index, 0, { type: 'folder', folder: item.folder });
		} else if (item.folderIndex === null) {
			// top-level mod
			items.splice(item.innerIndex, 0, { type: 'mod', mod: item.mod });
		} else {
			const folderItem = items[item.folderIndex];
			if (folderItem.type !== 'folder') {
				console.warn('Expected folder item at index', item.folderIndex);
				return;
			}
			folderItem.folder.mods.splice(item.innerIndex, 0, item.mod);
		}
	}

	function onDragEnd(event: any) {
		if (
			event.canceled ||
			draggingId === null ||
			draggingItem === null ||
			targetId === null ||
			dropState === null
		) {
			resetDragState();
			return;
		}

		const newItems = $state.snapshot(items);

		// remove the dragged item from its original location
		removeItem(newItems, draggingItem);

		let targetItem = findItem(newItems, targetId);
		if (!targetItem) {
			console.warn('Target item not found after removing dragged item');
			resetDragState();
			return;
		}

		if (dropState === 'folder') {
			// nested folders are not supported
			if (draggingItem.type !== 'mod') {
				console.warn('Dragging item is not a mod, cannot drop into folder');
				resetDragState();
				return;
			}

			if (targetItem.type === 'folder') {
				// drop into folder operation; move the dragged mod into the target folder
				targetItem.folder.mods.push(draggingItem.mod);
			} else {
				// create a new folder at the location of the target item,
				// and move both the dragged mod and the target mod into it

				// removing the dragged item from the list may have changed the location of the target item, so we need to recalculate it
				targetItem = findItem(newItems, targetId);
				if (targetItem === null || targetItem.type !== 'mod' || targetItem.folderIndex !== null) {
					console.warn('Target item not found or is not a mod after recalculation');
					resetDragState();
					return;
				}

				// replace the target item with a new folder
				newItems.splice(targetItem.innerIndex, 1);

				const folder: Folder = {
					id: crypto.randomUUID(),
					name: m.page_folderNew(),
					mods: [draggingItem.mod, targetItem.mod],
					isExpanded: false
				};

				newItems.splice(targetItem.innerIndex, 0, { type: 'folder', folder });
			}
		} else {
			// calculate the new ItemWithLocation for the dragged item based on the drop state
			let newItem: ItemWithLocation;

			const targetInnerIndex =
				targetItem.type === 'folder' ? targetItem.index : targetItem.innerIndex;

			if (draggingItem.type === 'folder') {
				newItem = {
					type: 'folder',
					index: adjustDropIndex(targetInnerIndex, dropState),
					folder: draggingItem.folder
				};
			} else {
				newItem = {
					type: 'mod',
					folderIndex: targetItem.type === 'folder' ? null : targetItem.folderIndex,
					innerIndex: adjustDropIndex(targetInnerIndex, dropState),
					mod: draggingItem.mod
				};
			}

			insertItem(newItems, newItem);
		}

		items = newItems;

		onLayoutChange?.(items);
		resetDragState();
	}

	function adjustDropIndex(index: number, state: DropState): number {
		if (state === 'after') {
			return index + 1;
		}
		return index;
	}

	function getDropState(id: DragIdentifier): DropState | null {
		if (targetId === null || draggingId === null) {
			return null;
		}

		if (id === targetId) {
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
					dropState={getDropState(item.folder.id)}
					contextItems={folderContextItems}
					ontoggle={(newState) => onFolderToggle?.(item.folder.id, newState)}
				>
					{#snippet mod({ mod, index: innerIndex })}
						{@render modItem(outerIndex * totalItems + innerIndex, mod)}
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
				<ProfileFolderCard folder={draggingItem.folder} />
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
		dropState={getDropState(mod.uuid)}
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
