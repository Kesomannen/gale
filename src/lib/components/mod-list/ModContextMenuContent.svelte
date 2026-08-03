<script lang="ts">
	import type { ContextItem, Mod, ModContextItem } from '$lib/types';
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';

	type Props = {
		mod: Mod;
		items: ModContextItem[];
		type: 'dropdown' | 'context';
		locked: boolean;
	};

	let { mod, items, type, locked }: Props = $props();

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

<ContextMenuContent {type} items={items.map(mapItem).filter((item) => item != null)} />
