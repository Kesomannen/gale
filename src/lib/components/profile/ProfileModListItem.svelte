<script lang="ts">
	import type { Mod } from '$lib/types';
	import { createSortable } from '@dnd-kit/svelte/sortable';
	import type { Snippet } from 'svelte';

	type Props = {
		id: string;
		index: number;
		reorderable?: boolean;
		ghost?: boolean;
		dropState?: 'before' | 'after' | 'folder' | null;
		children?: Snippet;
	};

	let { id, index, reorderable, ghost = false, dropState = null, children }: Props = $props();

	const sortable = createSortable({
		get id() {
			return id;
		},
		get index() {
			return index;
		},
		get disabled() {
			return !reorderable;
		},
		transition: {
			duration: 0
		}
	});
</script>

<div
	{id}
	{@attach sortable.attach}
	class={[
		'relative select-none',
		ghost && 'opacity-55',
		dropState === 'folder' && 'bg-accent-500/10 ring-accent-400 animate-pulse ring-2'
	]}
>
	{#if dropState === 'before'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 top-0 z-20 h-0.5 -translate-y-1/2 rounded-full"
		></div>
	{/if}

	{@render children?.()}

	{#if dropState === 'after'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 bottom-0 z-20 h-0.5 translate-y-1/2 rounded-full"
		></div>
	{/if}
</div>
