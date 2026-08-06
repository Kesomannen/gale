<script lang="ts">
	import type { Mod } from '$lib/types';
	import { createSortable } from '@dnd-kit/svelte/sortable';
	import type { Snippet } from 'svelte';

	type Props = {
		mod: Mod;
		index: number;
		reorderable?: boolean;
		isGhost?: boolean;
		dropLine?: 'before' | 'after' | null;
		children?: Snippet;
	};

	let { mod, index, reorderable, isGhost = false, dropLine = null, children }: Props = $props();

	const sortable = createSortable({
		get id() {
			return mod.uuid;
		},
		get index() {
			return index;
		},
		get data() {
			return { mod };
		},
		get disabled() {
			return !reorderable;
		},
		transition: {
			duration: 100,
			easing: 'cubic-bezier(0.2, 0, 0, 1)'
		}
	});
</script>

<div
	{@attach sortable.attach}
	id={mod.uuid}
	class={['relative select-none', isGhost && 'opacity-55']}
>
	{#if dropLine === 'before'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 top-0 z-20 h-0.5 -translate-y-1/2 rounded-full"
		></div>
	{/if}

	{@render children?.()}

	{#if dropLine === 'after'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 bottom-0 z-20 h-0.5 translate-y-1/2 rounded-full"
		></div>
	{/if}
</div>
