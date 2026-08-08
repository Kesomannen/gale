<script lang="ts" generics="T">
	import { ContextMenu } from 'bits-ui';
	import ModContextMenuContent from './ModContextMenuContent.svelte';
	import type { Mod, ModContextItem } from '$lib/types';
	import type { Snippet } from 'svelte';
	import { activeContextMenu } from '$lib/context';

	type Props = {
		id: string;
		subject: T;
		locked: boolean;
		contextItems: ModContextItem<T>[];
		children?: Snippet;
	};

	let { id, subject, children, locked, contextItems }: Props = $props();

	let contextMenuOpen = $state(false);

	$effect(() => {
		if ($activeContextMenu !== null && $activeContextMenu !== id) {
			contextMenuOpen = false;
		}
	});
</script>

<ContextMenu.Root
	bind:open={contextMenuOpen}
	onOpenChange={(open) => {
		if (open) {
			$activeContextMenu = id;
		} else {
			$activeContextMenu = null;
		}
	}}
>
	<ContextMenu.Trigger class="contents">
		{@render children?.()}
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" {locked} {subject} items={contextItems} />
</ContextMenu.Root>
