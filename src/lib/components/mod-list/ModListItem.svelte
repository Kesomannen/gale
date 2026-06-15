<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Spinner from '../ui/Spinner.svelte';
	import ModItemWithContext from './ModItemContext.svelte';
	import ModItem from './ModItem.svelte';
	import { formatModName, isOutdated, modIconSrc } from '$lib/util';

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
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		{onclick}
		role="button"
		tabindex="0"
		class={[
			'group flex w-full items-center gap-3 rounded-lg border p-2',
			selected ? 'border-primary-500 bg-primary-700' : 'hover:bg-primary-700 border-transparent'
		]}
	>
		<img src={modIconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
		<div class="shrink grow overflow-hidden text-left">
			<div class="flex items-center gap-1 overflow-hidden">
				<div class="shrink truncate pr-1 font-medium text-white">
					{formatModName(mod.name)}
				</div>
				{#if mod.isPinned}
					<Icon class="text-primary-400 shrink-0" icon="mdi:pin" />
				{/if}
				{#if mod.isDeprecated}
					<Icon class="shrink-0 text-red-500" icon="mdi:error" />
				{/if}
				{#if mod.isInstalled}
					<Icon class="text-accent-500 shrink-0" icon="mdi:check-circle" />
				{/if}
				{#if isOutdated(mod)}
					<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
				{/if}
			</div>

			{#if mod.description !== null}
				<div class="text-primary-400 truncate">
					{mod.description}
				</div>
			{/if}
		</div>

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
	</div>
</ModItemWithContext>
