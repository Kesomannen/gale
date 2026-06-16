<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemContext from './ModItemContext.svelte';
	import { formatModName, isOutdated, modIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		mod: Mod;
		selected?: boolean;
		onclick?: MouseEventHandler<HTMLDivElement>;
		children?: Snippet;
	};

	let { mod, selected, onclick, children }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	{onclick}
	role="button"
	tabindex="0"
	class={[
		'group text-primary-300 grid w-full grid-cols-[2fr_1fr_auto] items-center gap-2 rounded-lg border p-2 lg:grid-cols-[2fr_1fr_1fr_auto]',
		selected ? 'border-primary-500 bg-primary-700' : 'hover:bg-primary-700 border-transparent'
	]}
>
	<div class="flex items-center overflow-hidden">
		<img src={modIconSrc(mod)} alt={mod.name} class="mr-3 size-10 rounded-sm" />

		<div class={[mod.enabled ? 'text-white' : 'line-through', 'mr-2 shrink truncate font-medium']}>
			{formatModName(mod.name)}
		</div>

		{#if mod.isPinned}
			<Icon class="text-primary-400 mr-1 shrink-0" icon="mdi:pin" />
		{/if}
		{#if mod.isDeprecated}
			<Icon class="mr-1 shrink-0 text-red-500" icon="mdi:error" />
		{/if}
		{#if isOutdated(mod)}
			<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
		{/if}
	</div>

	<div class="hidden overflow-hidden lg:block">
		{mod.author}
	</div>

	<div>
		{mod.version}
	</div>

	{@render children?.()}
</div>
