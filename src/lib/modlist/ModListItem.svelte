<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { createEventDispatcher } from 'svelte';
	import { iconSrc } from '$lib/util';

	export let mod: Mod;
	export let isSelected: boolean;

	const dispatch = createEventDispatcher<{
		install: void;
	}>();

	$: descriptionClasses = isSelected
		? 'text-slate-300'
		: 'text-slate-400 group-hover:text-slate-300';
</script>

<button
	class="group flex w-full rounded-lg border p-2 {isSelected
		? 'border-slate-500 bg-slate-700'
		: 'border-transparent hover:bg-slate-700'}"
	on:click
>
	<img src={iconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
	<div class="shrink grow overflow-hidden pl-3 text-left">
		<div class="flex items-center gap-1 overflow-hidden">
			<div class="shrink truncate pr-1 font-medium text-white">
				{mod.name.replace(/_/g, ' ')}
			</div>
			{#if mod.isPinned}
				<Icon class="shrink-0 text-slate-400" icon="mdi:pin" />
			{/if}
			{#if mod.isDeprecated}
				<Icon class="shrink-0 text-red-500" icon="mdi:error" />
			{/if}
			{#if mod.isInstalled}
				<Icon class="text-accent-500 shrink-0" icon="mdi:check-circle" />
			{/if}
		</div>

		{#if mod.description !== null}
			<div class="truncate {descriptionClasses}">
				{mod.description}
			</div>
		{/if}
	</div>

	{#if !mod.isInstalled}
		<Button.Root
			class="bg-accent-600 hover:bg-accent-500 mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline"
			on:click={(evt) => {
				dispatch('install');
				evt.stopPropagation();
			}}
		>
			<Icon icon="mdi:download" />
		</Button.Root>
	{/if}
</button>
