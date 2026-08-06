<script lang="ts">
	import { DragDropProvider } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import { PointerActivationConstraints, PointerSensor } from '@dnd-kit/dom';
	import ProfileModListModItem from './ProfileModListModItem.svelte';
	import type { ListItem, Mod } from '$lib/types';
	import type { Snippet } from 'svelte';
	import VirtualList from '../ui/VirtualList.svelte';

	type Props = {
		items: ListItem[];
		mod: Snippet<[{ mod: Mod; index: number }]>;
		onmove?: (item: ListItem, fromIndex: number, toIndex: number) => void;
		reorderable?: boolean;
	};

	let { items = $bindable(), mod, onmove, reorderable = true }: Props = $props();

	type DragIdentifier = string | number;
	type DropSide = 'before' | 'after';

	let draggingId = $state<DragIdentifier | null>(null);
	let draggingIndex = $state<number | null>(null);
	let dropIndex = $state<number | null>(null);
	let dropSide = $state<DropSide | null>(null);

	function itemId(item: ListItem) {
		return item.type === 'folder' ? item.folder.id : item.mod.uuid;
	}

	function resetDragState() {
		draggingId = null;
		draggingIndex = null;
		dropIndex = null;
		dropSide = null;
	}

	function onDragStart(event: any) {
		const { source } = event.operation;

		if (!isSortable(source)) {
			resetDragState();
			return;
		}

		draggingId = source.id;
		draggingIndex = items.findIndex((item) => itemId(item) === source.id);
		dropIndex = draggingIndex;
		dropSide = 'after';
	}

	function onDragOver(event: any) {
		const { source, target } = event.operation;

		if (!isSortable(source) || !isSortable(target) || source.id === target.id) {
			return;
		}

		const fromIndex = items.findIndex((item) => itemId(item) === source.id);
		const targetIndex = items.findIndex((item) => itemId(item) === target.id);
		const targetElement = target.element as HTMLElement | undefined;

		if (fromIndex === -1 || targetIndex === -1) {
			return;
		}

		draggingId = source.id;
		draggingIndex = fromIndex;

		if (targetElement && event.nativeEvent instanceof PointerEvent) {
			const rect = targetElement.getBoundingClientRect();
			const isBefore = event.nativeEvent.clientY < rect.top + rect.height / 2;

			dropSide = isBefore ? 'before' : 'after';
			dropIndex = targetIndex;
			return;
		}

		dropSide = 'after';
		dropIndex = targetIndex;
	}

	function onDragEnd(event: any) {
		if (event.canceled || draggingIndex === null || dropIndex === null) {
			resetDragState();
			return;
		}

		const fromIndex = draggingIndex;
		const toIndex = getFinalDropIndex(fromIndex, dropIndex, dropSide);

		if (fromIndex === toIndex) {
			resetDragState();
			return;
		}

		const newItems = [...items];
		const [removed] = newItems.splice(fromIndex, 1);
		newItems.splice(toIndex, 0, removed);
		items = newItems;

		onmove?.(removed, fromIndex, toIndex);
		resetDragState();
	}

	function getFinalDropIndex(fromIndex: number, targetIndex: number, side: DropSide | null) {
		if (side === 'before') {
			return fromIndex < targetIndex ? targetIndex - 1 : targetIndex;
		}

		return fromIndex < targetIndex ? targetIndex : targetIndex + 1;
	}

	function getDropLine(index: number) {
		if (dropIndex === null || draggingIndex === null || dropIndex === draggingIndex) {
			return null;
		}

		if (dropIndex === index) {
			return dropSide;
		}

		return null;
	}
</script>

<DragDropProvider
	{onDragStart}
	{onDragOver}
	{onDragEnd}
	sensors={(defaults) => [
		...defaults.filter((sensor) => sensor !== PointerSensor),
		PointerSensor.configure({
			activationConstraints: [new PointerActivationConstraints.Distance({ value: 6 })]
		})
	]}
>
	<VirtualList {items} rowId={(item) => itemId(item)} itemHeight={58}>
		{#snippet children({ item, index })}
			{#if item.type !== 'folder'}
				<ProfileModListModItem
					mod={item.mod}
					{index}
					{reorderable}
					isGhost={draggingId === item.mod.uuid}
					dropLine={getDropLine(index)}
				>
					{@render mod({ mod: item.mod, index })}
				</ProfileModListModItem>
			{/if}
		{/snippet}
	</VirtualList>
</DragDropProvider>
