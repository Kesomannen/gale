<script lang="ts">
	import type { Mod } from '../../types';
	import type { MouseEventHandler } from 'svelte/elements';
	import { formatModName, isOutdated, modIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import type { ClassValue } from 'clsx';

	type Props = {
		mod: Mod;
		index?: number;
		class?: ClassValue;
		selected?: boolean;
		onclick?: MouseEventHandler<HTMLDivElement>;
		leading?: Snippet;
		trailing?: Snippet;
	};

	let { mod, index = 0, class: classProp, selected, onclick, leading, trailing }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	{onclick}
	role="button"
	tabindex="-1"
	class={[
		classProp,
		!mod.enabled && 'opacity-70',
		// can't use tailwind's odd: because the items are wrapped the virtual list item elements
		selected ? 'bg-primary-700' : index % 2 === 1 && 'bg-primary-900/30',
		selected ? 'border-primary-500' : 'hover:bg-primary-700 border-transparent',
		'group text-primary-300 mx-2 grid grid-cols-[auto_3fr_1fr_auto] items-center rounded-lg border p-2 lg:grid-cols-[auto_3fr_1fr_1fr_auto]'
	]}
>
	{#if leading}
		{@render leading()}
	{:else}
		<div></div>
	{/if}

	<div class="flex items-center overflow-hidden">
		<img src={modIconSrc(mod)} alt={mod.name} class="mr-3 size-12 rounded-md" />

		<div class="mr-2 shrink overflow-hidden">
			<div
				class={[mod.enabled ? 'text-white' : 'line-through', 'font-medium', 'flex items-center']}
			>
				<span class="mr-2 truncate">{formatModName(mod.name)}</span>

				{#if mod.isPinned}
					<Icon class="text-primary-400 mr-1 shrink-0" icon="mdi:pin" />
				{/if}
				{#if mod.isDeprecated}
					<Icon class="mr-1 shrink-0 text-yellow-500" icon="mdi:warning" />
				{/if}
				{#if isOutdated(mod)}
					<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
				{/if}
			</div>

			{#if mod.description}
				<div class="text-primary-400 truncate text-sm">
					{mod.description}
				</div>
			{/if}
		</div>
	</div>

	<div class="hidden overflow-hidden lg:block">
		{mod.author}
	</div>

	<div>
		{mod.version}
	</div>

	{#if trailing}
		{@render trailing()}
	{:else}
		<div></div>
	{/if}
</div>
