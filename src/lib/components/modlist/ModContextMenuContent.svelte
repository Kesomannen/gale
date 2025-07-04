<script lang="ts">
	import type { ContextItem, Mod, ModContextItem } from '$lib/types';
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';

	type Props = {
		mod: Mod;
		items: ModContextItem[];
		type: 'dropdown' | 'context';
		style: 'dark' | 'light';
		locked: boolean;
	};

	let { mod, items, type, style, locked }: Props = $props();

	function mapItem(modItem: ModContextItem): ContextItem | null {
		if (modItem.showFor && !modItem.showFor(mod, locked)) {
			return null;
		}

		return {
			label: modItem.label,
			icon: modItem.icon,
			onclick: () => modItem.onclick(mod),
			children: modItem
				.children?.(mod)
				?.map(mapItem)
				.filter((item) => item != null)
		};
	}
</script>

<ContextMenuContent {type} {style} items={items.map(mapItem).filter((item) => item != null)} />
