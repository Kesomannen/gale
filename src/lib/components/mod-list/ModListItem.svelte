<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Spinner from '../ui/Spinner.svelte';
	import ModItemWithContext from './ModItemContext.svelte';
	import ModItem from './ModItem.svelte';

	type Props = {
		mod: Mod;
		selected: boolean;
		locked: boolean;
		contextItems: ModContextItem[];
		onclick?: MouseEventHandler<HTMLDivElement>;
		oninstall?: () => void;
	};

	let { mod, selected: selected, locked, contextItems, onclick, oninstall }: Props = $props();

	let loading = $state(false);
</script>

<ModItemWithContext {mod} {locked} {contextItems}>
	<ModItem {mod} {selected} {onclick}>
		{#if !mod.isInstalled && !locked}
			<!-- svelte-ignore node_invalid_placement_ssr -->
			<!-- we're not using ssr -->
			<button
				class="bg-accent-600 hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline"
				disabled={loading}
				onclick={(evt) => {
					evt.stopPropagation();
					oninstall?.();
					loading = true;
				}}
			>
				{#if loading}
					<Spinner />
				{:else}
					<Icon icon="mdi:download" />
				{/if}
			</button>
		{/if}
	</ModItem>
</ModItemWithContext>
