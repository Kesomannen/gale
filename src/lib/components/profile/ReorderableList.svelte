<script lang="ts">
	import { DragDropProvider, DragOverlay } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import ReorderableMod from './ReorderableMod.svelte';
	import type { ListItem, Mod } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ReorderableFolder from './ReorderableFolder.svelte';
	import ModItem from '../mod-list/ModItem.svelte';
	import VirtualList from '../ui/VirtualList.svelte';

	type Props = {
		items: ListItem[];
		mod: Snippet<[{ mod: Mod }]>;
		onmove?: (item: ListItem, fromIndex: number, toIndex: number) => void;
		reorderable?: boolean;
	};

	let { items = $bindable(), mod, onmove, reorderable = true }: Props = $props();

	let dragging: ListItem | null = $state(null);
	let hovering: ListItem | null = $state(null);

	function createFolderItem(children: ListItem[]): ListItem {
		return {
			type: 'folder',
			folder: {
				id: crypto.randomUUID(),
				children
			}
		};
	}

	function itemId(item: ListItem) {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
	}

	function onDragStart(event: any) {
		dragging = event.operation.source.id;
	}

	function onDragOver(event: any) {
		const { source, target } = event.operation;

		if (!isSortable(source) || !isSortable(target) || source.id === target.id) {
			return;
		}

		const fromIndex = items.findIndex((item) => itemId(item) === source.id);
		const toIndex = items.findIndex((item) => itemId(item) === target.id);

		if (fromIndex === -1 || toIndex === -1) {
			return;
		}

		const newItems = [...items];
		const [removed] = newItems.splice(fromIndex, 1);
		newItems.splice(toIndex, 0, removed);
		items = newItems;

		onmove?.(removed, fromIndex, toIndex);
	}

	function onDragEnd(event: any) {}
</script>

<DragDropProvider {onDragStart} {onDragOver} {onDragEnd}>
	<VirtualList {items} rowId={(item) => itemId(item)} itemHeight={58}>
		{#snippet children({ item, index })}
			{@const hovered = item === hovering}

			{#if item.type === 'folder'}
				<ReorderableFolder folder={item.folder} {index} {hovered} />
			{:else}
				<ReorderableMod mod={item.mod} {index} {hovered} disabled={!reorderable}>
					{@render mod({ mod: item.mod })}
				</ReorderableMod>
			{/if}
		{/snippet}
	</VirtualList>

	<DragOverlay dropAnimation={{ duration: 150, easing: 'cubic-bezier(0.33, 1, 0.68, 1)' }}>
		{#snippet children(source)}
			{#if source.data.mod}
				<ModItem mod={source.data.mod} hideInstalledIcon />
			{:else if source.data.folder}
				<div
					class="bg-primary-950 border-primary-800 text-primary-200 rounded-xl border px-4 py-3 shadow-lg"
				>
					Folder
				</div>
			{/if}
		{/snippet}
	</DragOverlay>
</DragDropProvider>
