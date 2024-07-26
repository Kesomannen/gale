<script lang="ts">
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';
	import { activeGame } from '$lib/stores';
	import { readFile } from '@tauri-apps/plugin-fs';
	import { convertFileSrc } from '@tauri-apps/api/core';

	export let mod: Mod;
	export let isSelected: boolean;
	export let showInstalledIcon: boolean;
	export let draggable = false;

	let img: HTMLImageElement;
	let iconSrc: string;

	$: {
		if (mod.type === 'remote') {
			iconSrc = mod.icon!;
		} else {
			iconSrc = mod.icon ? convertFileSrc(mod.icon) : 'games/' + $activeGame?.id + '.webp';
		}
	}
</script>

<button
	class="flex border border-slate-500 rounded-lg p-2 group w-full {isSelected
		? 'bg-slate-700'
		: 'hover:bg-slate-700 border-opacity-0'}"
	data-uuid={mod.uuid}
	on:click
	on:dragstart
	on:dragover
	on:drag
	on:dragend
	{draggable}
>
	<img bind:this={img} src={iconSrc} alt={mod.name} class="w-12 h-12 rounded-md" />
	<div class="pl-3 overflow-hidden flex-grow flex-shrink align-middle text-left">
		<span
			class="font-semibold {mod.enabled === false ? 'line-through text-slate-300' : 'text-white'}"
		>
			{mod.name}
		</span>
		<span
			class="font-light px-1 {mod.enabled === false
				? 'line-through text-slate-500'
				: 'text-slate-400'}"
		>
			{mod.version ?? ''}
		</span>
		{#if false && showInstalledIcon}
			<Icon class="text-green-500 inline mb-1" icon="mdi:check" />
		{/if}
		{#if mod.isDeprecated}
			<Icon class="text-red-500 inline mb-1" icon="mdi:error" />
		{/if}
		{#if isOutdated(mod)}
			<Icon class=" text-blue-500 inline mb-1.5" icon="mdi:arrow-up-circle" />
		{/if}
		<div
			class="truncate {mod.enabled === false ? 'line-through text-slate-500' : 'text-slate-300/80'}"
		>
			{mod.description ?? ''}
		</div>
	</div>

	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<div class="contents" on:click={(evt) => evt.stopPropagation()} role="none">
		<slot isInstalled={false} />
	</div>
</button>
