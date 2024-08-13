<script lang="ts">
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';
	import { readFile } from '@tauri-apps/plugin-fs';
	import { activeGame } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';

	export let mod: Mod;
	export let isSelected: boolean;
	export let draggable = false;

	let imgSrc: string;
	let isInstalled: boolean = false;

	$: {
		if (mod.type === 'remote') {
			imgSrc = mod.icon!;
		} else {
			if (mod.icon) {
				imgSrc = '';
				loadLoadIcon(mod.icon);
			} else {
				imgSrc = `games/${$activeGame?.id}.webp`;
			}
		}
	}

	$: {
		invokeCommand<boolean>('is_mod_installed', { uuid: mod.uuid }).then(
			(result) => (isInstalled = result)
		);
	}

	async function loadLoadIcon(path: string) {
		try {
			let data = await readFile(path);
			let blob = new Blob([data], { type: 'image/png' });
			imgSrc = URL.createObjectURL(blob);
		} catch (e) {
			console.error(e);
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
	<img src={imgSrc} alt={mod.name} class="w-12 h-12 rounded-md" />
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
		{#if mod.isPinned}
			<Icon class="text-slate-400 inline mb-1" icon="mdi:pin" />
		{/if}
		{#if mod.isDeprecated}
			<Icon class="text-red-500 inline mb-1" icon="mdi:error" />
		{/if}
		{#if isOutdated(mod)}
			<Icon class=" text-green-500 inline mb-1.5" icon="mdi:arrow-up-circle" />
		{/if}
		{#if isInstalled}
			<Icon class="text-green-500 inline mb-1.5" icon="mdi:check-circle" />
		{/if}
		<div
			class="truncate {mod.enabled === false ? 'line-through text-slate-500' : 'text-slate-300/80'}"
		>
			{mod.description ?? ''}
		</div>
	</div>

	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<div class="contents" on:click={(evt) => evt.stopPropagation()} role="none">
		<slot {isInstalled} />
	</div>
</button>
