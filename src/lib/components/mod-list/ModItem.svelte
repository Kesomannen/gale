<script lang="ts">
	import type { Mod } from '../../types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc, isOutdated } from '$lib/util';
	import type { MouseEventHandler } from 'svelte/elements';
	import type { Snippet } from 'svelte';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		children?: Snippet;
	};

	let { mod, isSelected, onclick, children }: Props = $props();

	let descriptionClasses = $derived(
		mod.enabled === false
			? 'text-primary-500 line-through'
			: isSelected
				? 'text-primary-300'
				: 'text-primary-400 group-hover:text-primary-300'
	);
</script>

<div
	{onclick}
	class={[
		'group flex w-full rounded-lg border p-2',
		isSelected ? 'border-primary-500 bg-primary-700' : 'hover:bg-primary-700 border-transparent'
	]}
>
	<img src={modIconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
	<div class="shrink grow overflow-hidden pl-3 text-left">
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
			<div class="truncate {descriptionClasses}">
				{mod.description}
			</div>
		{/if}
	</div>

	{@render children?.()}
</div>
