<script lang="ts">
	import type { Folder, Mod, ModContextItem } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ProfileModListItem from './ProfileModListItem.svelte';
	import ProfileFolderCard from './ProfileFolderCard.svelte';
	import ModItemContext from '../mod-list/ModItemContext.svelte';
	import ModSwitch from './ModSwitch.svelte';

	type Props = {
		folder: Folder;
		index: number;
		reorderable?: boolean;
		locked?: boolean;
		contextItems: ModContextItem<Folder>[];
		mod: Snippet<[{ mod: Mod; index: number }]>;
		ontoggle?: (newState: boolean) => void;
		dropState?: 'before' | 'after' | 'folder' | null;
		ghost?: boolean;
	};

	let {
		folder = $bindable(),
		index,
		reorderable,
		locked = false,
		contextItems,
		mod: modSnippet,
		ontoggle,
		dropState = null,
		ghost
	}: Props = $props();

	const enabled = $derived(
		folder.mods.length === 0 || folder.mods.some((mod) => mod.enabled ?? true)
	);
</script>

<ProfileModListItem id={folder.id} {index} {reorderable} {dropState} {ghost}>
	<ModItemContext id={folder.id} subject={folder} {locked} {contextItems}>
		<ProfileFolderCard {index} {folder} mod={modSnippet}>
			{#snippet trailing()}
				<ModSwitch {enabled} {locked} ontoggle={(newState) => ontoggle?.(newState)} />
			{/snippet}
		</ProfileFolderCard>
	</ModItemContext>
</ProfileModListItem>
