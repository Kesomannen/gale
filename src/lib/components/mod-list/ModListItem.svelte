<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Spinner from '../ui/Spinner.svelte';
	import ModItemWithContext from './ModItemWithContext.svelte';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		locked: boolean;
		contextItems: ModContextItem[];
		onclick?: MouseEventHandler<HTMLButtonElement>;
		oninstall?: () => void;
	};

	let { mod, isSelected, locked, contextItems, onclick, oninstall }: Props = $props();

	let loading = $state(false);
</script>

<ModItemWithContext {mod} {isSelected} {onclick} {locked} {contextItems}>
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
</ModItemWithContext>
