<script lang="ts">
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';
	import { readFile } from '@tauri-apps/plugin-fs';
	import { activeGame } from '$lib/stores';
	import { Switch } from 'bits-ui';
	import { createEventDispatcher } from 'svelte';

	export let mod: Mod;
	export let index: number;
	export let isSelected: boolean;
	export let reorderable: boolean;

	const dispatch = createEventDispatcher<{
		toggle: boolean;
	}>();

	let imgSrc: string;

	$: {
		if (mod.type === 'remote') {
			imgSrc = mod.icon!;
		} else {
			if (mod.icon === null) {
				imgSrc = `games/${$activeGame?.id}.webp`;
			} else {
				imgSrc = '';
				loadLoadIcon(mod.icon);
			}
		}
	}

	$: descriptionClasses =
		mod.enabled === false
			? 'text-slate-500 line-through'
			: isSelected
				? 'text-slate-300'
				: 'text-slate-400 group-hover:text-slate-300';

	async function loadLoadIcon(path: string) {
		try {
			let data = await readFile(path);
			let blob = new Blob([data], { type: 'image/png' });
			imgSrc = URL.createObjectURL(blob);
		} catch (err) {
			console.error(err);
		}
	}
</script>

<button
	class="group flex w-full items-center rounded-lg border border-slate-500 p-2 {isSelected
		? 'bg-slate-700'
		: 'border-opacity-0 hover:bg-slate-700'}"
	data-uuid={mod.uuid}
	data-index={index}
	draggable="true"
	on:click
	on:dragstart
	on:dragover
	on:dragend
>
	<img src={imgSrc} alt={mod.name} class="size-12 rounded" />
	<div class="flex-shrink flex-grow overflow-hidden pl-3 text-left">
		<div class="flex items-center gap-1 overflow-hidden">
			<div
				class="flex-shrink truncate font-medium {mod.enabled === false
					? 'text-slate-300 line-through'
					: 'text-white'}"
			>
				{mod.name.replace(/_/g, ' ')}
			</div>
			<div class="px-1 {descriptionClasses}">
				{mod.version ?? '?.?.?'}
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
		</div>

		{#if mod.description !== null}
			<div class="truncate {descriptionClasses}">
				{mod.description}
			</div>
		{/if}
	</div>

	{#if reorderable}
		<Icon
			icon="material-symbols:drag-indicator"
			class="mr-3 flex-shrink-0 cursor-move text-2xl text-slate-400"
		/>
	{/if}

	<!-- make sure click events don't propagate and cause the mod to be selected -->
	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div class="contents" on:click={(evt) => evt.stopPropagation()}>
		<Switch.Root
			checked={mod.enabled ?? true}
			onCheckedChange={(newState) => dispatch('toggle', newState)}
			class="group mr-1 flex h-6 w-12 flex-shrink-0 rounded-full bg-slate-600 px-1 py-1 hover:bg-slate-500 data-[state=checked]:bg-green-700 data-[state=checked]:hover:bg-green-600"
		>
			<Switch.Thumb
				class="pointer-events-none h-full w-4 rounded-full bg-slate-300 transition-transform duration-75 ease-out hover:bg-slate-200 data-[state=checked]:translate-x-6 data-[state=checked]:bg-green-200 data-[state=checked]:group-hover:bg-green-100"
			/>
		</Switch.Root>
	</div>
</button>
