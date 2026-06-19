<script lang="ts">
	import { DragDropProvider, DragOverlay } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import ReorderableMod from './ReorderableMod.svelte';
	import type { ListItem, Mod } from '$lib/types';
	import type { Snippet } from 'svelte';
	import VirtualList from '../ui/VirtualList.svelte';
	import ProfileModListItemNoContext from '../mod-list/ProfileModListItemNoContext.svelte';

	type Props = {
		items: ListItem[];
		mod: Snippet<[{ mod: Mod }]>;
		onmove?: (item: ListItem, fromIndex: number, toIndex: number) => void;
		reorderable?: boolean;
	};

	let { items = $bindable(), mod, onmove, reorderable = true }: Props = $props();

	let hovering: ListItem | null = $state(null);

	function itemId(item: ListItem) {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
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
</script>

<DragDropProvider {onDragOver}>
	<VirtualList {items} rowId={(item) => itemId(item)} itemHeight={58}>
		{#snippet children({ item, index })}
			{@const hovered = item === hovering}

			{#if item.type === 'folder'}
				<!-- <ReorderableFolder folder={item.folder} {index} {hovered} /> -->
			{:else}
				<ReorderableMod mod={item.mod} {index} {hovered} disabled={!reorderable}>
					{@render mod({ mod: item.mod })}
				</ReorderableMod>
			{/if}
		{/snippet}
	</VirtualList>

	<DragOverlay dropAnimation={{ duration: 150, easing: 'cubic-bezier(0.33, 1, 0.68, 1)' }}>
		{#snippet children(source)}
			<ProfileModListItemNoContext mod={source.data.mod} />
		{/snippet}
	</DragOverlay>
</DragDropProvider>
