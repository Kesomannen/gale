<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Spinner from '../ui/Spinner.svelte';
	import ModItemWithContext from './ModItemContext.svelte';
	import { formatModName, modIconSrc, shortenNum, timeSince } from '$lib/util';

	type Props = {
		mod: Mod;
		combinedDownloads?: number;
		isInstalled?: boolean;
		selected?: boolean;
		locked?: boolean;
		contextItems: ModContextItem[];
		onclick?: MouseEventHandler<HTMLDivElement>;
		oninstall?: () => Promise<void>;
	};

	let {
		mod,
		combinedDownloads,
		isInstalled = false,
		selected = false,
		locked = false,
		contextItems,
		onclick,
		oninstall
	}: Props = $props();

	let loading = $state(false);

	let downloads = $derived(combinedDownloads ?? mod.downloads ?? 0);
</script>

<ModItemWithContext uuid={mod.uuid} {mod} {locked} {contextItems}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		{onclick}
		role="button"
		tabindex="0"
		class={[
			'group text-primary-500 dark:text-primary-400 my-1 flex items-center gap-4 rounded-lg border p-3',
			selected
				? 'border-primary-500 dark:border-primary-500 dark:bg-primary-700 bg-primary-200'
				: 'border-primary-200 hover:border-primary-300 dark:hover:bg-primary-700 dark:border-primary-700 dark:hover:border-primary-600 hover:bg-primary-200'
		]}
	>
		<img src={modIconSrc(mod)} alt={mod.name} class="size-18 rounded-lg" />

		<div class="shrink grow overflow-hidden text-left">
			<div class="flex items-center gap-1 overflow-hidden">
				<div class="text-primary-900 truncate pr-1 text-lg font-medium dark:text-white">
					{formatModName(mod.name)}
				</div>
				{#if mod.author !== null}
					<div class="text-primary-600 dark:text-primary-300 truncate pr-2">
						{mod.author}
					</div>
				{/if}
				{#if mod.isPinned}
					<Icon class="text-primary-500 dark:text-primary-400 shrink-0" icon="mdi:pin" />
				{/if}
				{#if mod.isDeprecated}
					<Icon class="shrink-0 text-yellow-500" icon="mdi:warning" />
				{/if}
				{#if isInstalled}
					<Icon class="text-accent-600 dark:text-accent-500 shrink-0" icon="mdi:check-circle" />
				{/if}
			</div>

			{#if mod.description !== null}
				<div class="line-clamp-1 text-ellipsis lg:line-clamp-2">
					{mod.description}
				</div>
			{/if}

			<div class="mt-1 flex flex-wrap items-center gap-1">
				<Icon class="shrink-0" icon="mdi:download-outline" />
				<span class="mr-4">{shortenNum(downloads)}</span>
				{#if mod.lastUpdated}
					<Icon class="shrink-0" icon="mdi:clock-outline" />
					<span class="mr-2">{timeSince(new Date(mod.lastUpdated))}</span>
				{/if}
			</div>
		</div>

		{#if !isInstalled && !locked}
			<button
				class={[
					'bg-accent-600 hover:bg-accent-500 disabled:bg-primary-600 dark:disabled:text-primary-300 mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline'
				]}
				disabled={loading}
				onclick={async (evt) => {
					evt.stopPropagation();
					if (!oninstall) return;

					loading = true;
					try {
						await oninstall();
					} finally {
						loading = false;
					}
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
