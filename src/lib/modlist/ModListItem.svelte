<script lang="ts">
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';
	import { readFile } from '@tauri-apps/plugin-fs';
	import { activeGame } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';

	export let mod: Mod;
	export let isSelected: boolean;
	export let isInstalled: boolean;
	export let showInstalledIcon: boolean;
	export let draggable = false;

	let imgSrc: string;

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
	class="group flex w-full rounded-lg border border-slate-500 p-2 {isSelected
		? 'bg-slate-700'
		: 'border-opacity-0 hover:bg-slate-700'}"
	data-uuid={mod.uuid}
	on:click
	on:dragstart
	on:dragover
	on:drag
	on:dragend
	{draggable}
>
	<img src={imgSrc} alt={mod.name} class="size-12 rounded-md" />
	<div class="flex-shrink flex-grow overflow-hidden pl-3 text-left">
		<div class="flex items-center overflow-hidden">
			<div
				class="flex-shrink truncate font-semibold {mod.enabled === false
					? 'text-slate-300 line-through'
					: 'text-white'}"
			>
				{mod.name}
			</div>
			<div
				class="px-2 {mod.enabled === false
					? 'text-slate-500 line-through'
					: 'text-slate-400 hover:text-slate-300'}"
			>
				{mod.version ?? ''}
			</div>
			{#if mod.isPinned}
				<Icon class="flex-shrink-0 text-slate-400" icon="mdi:pin" />
			{/if}
			{#if mod.isDeprecated}
				<Icon class="flex-shrink-0 text-red-500" icon="mdi:error" />
			{/if}
			{#if isOutdated(mod)}
				<Icon class="flex-shrink-0 text-green-500" icon="mdi:arrow-up-circle" />
			{/if}
			{#if isInstalled && showInstalledIcon}
				<Icon class="flex-shrink-0 text-green-500" icon="mdi:check-circle" />
			{/if}
		</div>

		<div
			class="truncate {mod.enabled === false
				? 'text-slate-500 line-through'
				: 'text-slate-400 group-hover:text-slate-300'}"
		>
			{mod.description ?? ''}
		</div>
	</div>

	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<div class="contents" on:click={(evt) => evt.stopPropagation()} role="none">
		<slot {isInstalled} />
	</div>
</button>
