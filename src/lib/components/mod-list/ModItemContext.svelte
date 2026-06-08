<script lang="ts">
	import { ContextMenu } from 'bits-ui';
	import ModContextMenuContent from './ModContextMenuContent.svelte';
	import type { Mod, ModContextItem } from '$lib/types';
	import type { Snippet } from 'svelte';
	import { activeContextMenu } from '$lib/context';

	type Props = {
		mod: Mod;
		locked: boolean;
		contextItems: ModContextItem[];
		children?: Snippet;
	};

	let { mod, children, locked, contextItems }: Props = $props();

	let contextMenuOpen = $state(false);

	$effect(() => {
		if ($activeContextMenu !== null && $activeContextMenu !== mod.uuid) {
			contextMenuOpen = false;
		}
	});
</script>

<ContextMenu.Root
	bind:open={contextMenuOpen}
	onOpenChange={(open) => {
		if (open) {
			$activeContextMenu = mod.uuid;
		} else {
			$activeContextMenu = null;
		}
	}}
>
	<ContextMenu.Trigger class="contents">
		{@render children?.()}
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>
