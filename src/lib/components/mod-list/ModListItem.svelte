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
	import translation from '$lib/state/translation.svelte';

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

	let displayName = $derived(translation.getDisplayName(mod.uuid, formatModName(mod.name)));
	let displayDescription = $derived(translation.getDisplayDescription(mod.uuid, mod.description));
</script>

<ModItemWithContext {mod} {locked} {contextItems}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		{onclick}
		role="button"
		tabindex="0"
		class={[
			'group my-1 grid grid-cols-[auto_1fr] items-center gap-4 rounded-lg border p-3',
			selected
				? 'border-primary-500 bg-primary-700'
				: 'hover:bg-primary-700 border-primary-700 hover:border-primary-600'
		]}
	>
		<img src={modIconSrc(mod)} alt={mod.name} class="size-18 rounded-lg" />

		<div class="shrink grow overflow-hidden text-left">
			<div class="flex items-center gap-1 overflow-hidden">
				<div class="shrink pr-1 text-lg font-medium text-white">
					{displayName}
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
				{#if translation.isTranslating(mod.uuid)}
					<Icon class="text-primary-400 shrink-0 animate-spin" icon="mdi:loading" />
				{/if}
			</div>

			{#if displayDescription !== null}
				<div class="text-primary-300 line-clamp-1 text-sm text-ellipsis lg:line-clamp-2">
					{displayDescription}
				</div>
			{/if}

			<div class="text-primary-300 mt-2 flex items-center gap-1 text-sm">
				{#if mod.rating !== null}
					<Icon class="shrink-0" icon="mdi:star-outline" />
					<span class="mr-4">{shortenNum(mod.rating)}</span>
				{/if}
				{#if mod.downloads !== null}
					<Icon class="shrink-0" icon="mdi:download-outline" />
					<span class="mr-4">{shortenNum(mod.downloads)}</span>
				{/if}
				{#if mod.lastUpdated}
					<Icon class=" shrink-0" icon="mdi:clock-outline" />
					<span class="">{timeSince(new Date(mod.lastUpdated))}</span>
				{/if}
			</div>
		</div>

		<!-- {#if !mod.isInstalled && !locked}
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
		{/if} -->
	</div>
</ModItemWithContext>
