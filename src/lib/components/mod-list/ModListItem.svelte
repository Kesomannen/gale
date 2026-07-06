<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Spinner from '../ui/Spinner.svelte';
	import ModItemWithContext from './ModItemContext.svelte';
	import {
		formatModName,
		isOutdated,
		modIconSrc,
		shortenFileSize,
		shortenNum,
		timeSince
	} from '$lib/util';

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
			'group text-primary-400 my-1 flex items-center gap-4 rounded-lg border p-3',
			selected
				? 'border-primary-500 bg-primary-700'
				: 'hover:bg-primary-700 border-primary-700 hover:border-primary-600'
		]}
	>
		<img src={modIconSrc(mod)} alt={mod.name} class="size-18 rounded-lg" />

		<div class="shrink grow overflow-hidden text-left">
			<div class="flex items-center gap-1 overflow-hidden">
				<div class="shrink pr-1 text-lg font-medium text-white">
					{formatModName(mod.name)}
				</div>
				{#if mod.author !== null}
					<div class="text-primary-300 truncate pr-2">
						{mod.author}
					</div>
				{/if}
				{#if mod.isPinned}
					<Icon class="text-primary-400 shrink-0" icon="mdi:pin" />
				{/if}
				{#if mod.isDeprecated}
					<Icon class="shrink-0 text-yellow-500" icon="mdi:warning" />
				{/if}
				{#if mod.isInstalled}
					<Icon class="text-accent-500 shrink-0" icon="mdi:check-circle" />
				{/if}
				{#if isOutdated(mod)}
					<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
				{/if}
			</div>

			{#if mod.description !== null}
				<div class="line-clamp-1 text-ellipsis lg:line-clamp-2">
					{mod.description}
				</div>
			{/if}

			<div class="mt-1 flex flex-wrap items-center gap-1">
				{#if mod.downloads !== null}
					<Icon class="shrink-0" icon="mdi:download-outline" />
					<span class="mr-4">{shortenNum(mod.downloads)}</span>
				{/if}
				{#if mod.lastUpdated}
					<Icon class="shrink-0" icon="mdi:clock-outline" />
					<span class="mr-2">{timeSince(new Date(mod.lastUpdated))}</span>
				{/if}

				{#each mod.categories?.slice(0, 3) as category (category)}
					<span
						class={[
							selected ? 'bg-primary-600' : 'bg-primary-700 group-hover:bg-primary-600',
							'text-primary-300 rounded-full px-2 py-1 text-xs'
						]}
					>
						{category}
					</span>
				{/each}
			</div>
		</div>

		{#if !mod.isInstalled && !locked}
			<button
				class={[
					'bg-accent-600 hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline'
				]}
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
