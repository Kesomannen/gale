<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import { Switch } from 'bits-ui';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemContext from '../mod-list/ModItemContext.svelte';
	import ProfileModCard from './ProfileModCard.svelte';
	import type { ClassValue } from 'clsx';
	import ModSwitch from './ModSwitch.svelte';

	type Props = {
		mod: Mod;
		index?: number;
		selected: boolean;
		contextItems: ModContextItem[];
		locked: boolean;
		class?: ClassValue;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLDivElement>;
	};

	let {
		mod,
		index,
		selected,
		contextItems,
		locked,
		class: classProp,
		ontoggle,
		onclick
	}: Props = $props();
</script>

<ModItemContext {mod} {locked} {contextItems}>
	<ProfileModCard {mod} {selected} {index} {onclick} class={classProp}>
		{#snippet trailing()}
			<ModSwitch enabled={mod.enabled ?? true} {locked} {ontoggle} />
		{/snippet}
	</ProfileModCard>
</ModItemContext>
