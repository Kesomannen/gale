<script lang="ts">
	import { createSortable } from '@dnd-kit/svelte/sortable';
	import type { Folder, ListItem } from '$lib/types';

	type Props = {
		folder: Folder;
		index: number;
		hovered: boolean;
	};

	let { folder, index, hovered = false }: Props = $props();

	const sortable = createSortable({
		get id() {
			return folder.id;
		},
		get index() {
			return index;
		},
		get data() {
			return { folder };
		},
		transition: {
			duration: 150,
			easing: 'cubic-bezier(0.33, 1, 0.68, 1)'
		}
	});

	function childLabel(item: ListItem) {
		return item.type === 'folder' ? `Folder (${item.folder.children.length})` : item.mod.name;
	}
</script>

<li
	{@attach sortable.attach}
	id={folder.id}
	class={[
		'rounded-xl border px-4 py-3 transition',
		sortable.isDragging && 'opacity-40',
		sortable.isDropTarget
			? 'border-accent-600 bg-accent-950/20 text-accent-100'
			: 'border-primary-800 bg-primary-950/40 text-primary-200'
	]}
>
	Folder
</li>
