<script lang="ts">
	import type { Mod } from '$lib/types';
	import { createSortable } from '@dnd-kit/svelte/sortable';
	import type { Snippet } from 'svelte';

	type Props = {
		mod: Mod;
		index: number;
		hovered: boolean;
		disabled?: boolean;
		children?: Snippet;
	};

	let { mod, index, hovered, disabled, children }: Props = $props();

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
			return disabled;
		},
		transition: {
			duration: 150,
			easing: 'cubic-bezier(0.33, 1, 0.68, 1)'
		}
	});
</script>

<li {@attach sortable.attach} id={mod.uuid} class={[sortable.isDragging && 'opacity-40']}>
	{@render children?.()}
</li>
