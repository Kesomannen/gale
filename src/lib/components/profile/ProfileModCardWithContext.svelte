<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import { Switch } from 'bits-ui';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemContext from '../mod-list/ModItemContext.svelte';
	import ProfileModCard from './ProfileModCard.svelte';
	import type { ClassValue } from 'clsx';
	import ModSwitch from './ModSwitch.svelte';
	import ProfileModListItem from './ProfileModListItem.svelte';

	type Props = {
		mod: Mod;
		index: number;
		reorderable?: boolean;
		selected: boolean;
		contextItems: ModContextItem<Mod>[];
		locked: boolean;
		class?: ClassValue;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLDivElement>;
		ghost?: boolean;
		dropState?: 'before' | 'after' | 'folder' | null;
	};

	let {
		mod,
		index,
		reorderable,
		selected,
		contextItems,
		locked,
		class: classProp,
		ontoggle,
		onclick,
		ghost,
		dropState = null
	}: Props = $props();
</script>

<ProfileModListItem {ghost} {index} {dropState} {reorderable} id={mod.uuid}>
	<ModItemContext id={mod.uuid} subject={mod} {locked} {contextItems}>
		<ProfileModCard {mod} {selected} {index} {onclick} class={classProp}>
			{#snippet trailing()}
				<ModSwitch enabled={mod.enabled ?? true} {locked} {ontoggle} />
			{/snippet}
		</ProfileModCard>
	</ModItemContext>
</ProfileModListItem>
