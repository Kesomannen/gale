<script lang="ts">
	import { DragDropProvider, DragOverlay } from '@dnd-kit/svelte';
	import { isSortable } from '@dnd-kit/svelte/sortable';
	import ReorderableMod from './ReorderableMod.svelte';
	import type { Mod } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ModItem from '../mod-list/ModItem.svelte';

	type Props = {
		mods: Mod[];
		item: Snippet<[{ mod: Mod; index: number; isSelected: boolean }]>;
	};

	let { mods = $bindable(), item }: Props = $props();

	let snapshot: Mod[] = [];

	function onDragStart() {
		snapshot = mods.slice();
	}

	function onDragOver(event: any) {
		const { source, target } = event.operation;

		if (isSortable(source) && isSortable(target)) {
			const fromIndex = mods.findIndex((mod) => mod.uuid === source.id);
			const toIndex = mods.findIndex((mod) => mod.uuid === target.id);

			if (fromIndex !== -1 && toIndex !== -1 && fromIndex !== toIndex) {
				const newMods = [...mods];
				const [removed] = newMods.splice(fromIndex, 1);
				newMods.splice(toIndex, 0, removed);
				mods = newMods;
			}
		}
	}

	function onDragEnd(event: any) {
		if (event.canceled) mods = snapshot;
	}
</script>

<DragDropProvider {onDragStart} {onDragOver} {onDragEnd}>
	<ul class="overflow-y-auto">
		{#each mods as mod, index (mod.uuid)}
			<ReorderableMod {mod} {index}>
				{@render item({ mod, index, isSelected: false })}
			</ReorderableMod>
		{/each}
	</ul>

	<DragOverlay>
		{#snippet children(source)}
			<ModItem mod={source.data.mod} isSelected={false} />
		{/snippet}
	</DragOverlay>
</DragDropProvider>
