<script lang="ts" generics="T">
	import { ContextMenu } from 'bits-ui';
	import type { Mod, ModContextItem } from '$lib/types';
	import type { Snippet } from 'svelte';
	import { activeContextMenu } from '$lib/context';
	import { resolveModContextItems } from '$lib/util';
	import ContextMenuContent from '../ui/ContextMenuContent.svelte';

	type Props = {
		uuid: string;
		mod: T;
		locked: boolean;
		contextItems: ModContextItem<T>[];
		children?: Snippet;
	};

	let { uuid, mod, children, locked, contextItems }: Props = $props();

	let contextMenuOpen = $derived($activeContextMenu === uuid);
</script>

<ContextMenu.Root
	open={contextMenuOpen}
	onOpenChange={(newOpen) => {
		if (newOpen) {
			$activeContextMenu = uuid;
		} else {
			$activeContextMenu = null;
		}
	}}
>
	<ContextMenu.Trigger class="contents">
		{@render children?.()}
	</ContextMenu.Trigger>
	<ContextMenuContent type="context" items={resolveModContextItems(contextItems, mod, locked)} />
</ContextMenu.Root>
