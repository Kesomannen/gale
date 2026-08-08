<script lang="ts" generics="T">
	import type { ContextItem, Mod, ModContextItem } from '$lib/types';
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';

	type Props = {
		subject: T;
		items: ModContextItem<T>[];
		type: 'dropdown' | 'context';
		locked: boolean;
	};

	let { subject, items, type, locked }: Props = $props();

	function mapItem(modItem: ModContextItem<T>): ContextItem | null {
		if (modItem.showFor && !modItem.showFor(subject, locked)) {
			return null;
		}

		return {
			label: modItem.label,
			icon: modItem.icon,
			onclick: () => modItem.onclick(subject),
			children: modItem
				.children?.(subject)
				?.map(mapItem)
				.filter((item) => item != null)
		};
	}
</script>

<ContextMenuContent {type} items={items.map(mapItem).filter((item) => item != null)} />
