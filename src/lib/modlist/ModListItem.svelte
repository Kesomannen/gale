<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { createEventDispatcher } from 'svelte';

	export let mod: Mod;
	export let isSelected: boolean;

	const dispatch = createEventDispatcher<{
		install: Mod;
	}>();

	$: descriptionClasses = isSelected
		? 'text-slate-300'
		: 'text-slate-400 group-hover:text-slate-300';
</script>

<button
	class="group flex w-full rounded-lg border border-slate-500 p-2 {isSelected
		? 'bg-slate-700'
		: 'border-opacity-0 hover:bg-slate-700'}"
	on:click
>
	<img src={mod.icon} alt={mod.name} class="size-12 rounded" />
	<div class="flex-shrink flex-grow overflow-hidden pl-3 text-left">
		<div class="flex items-center gap-1 overflow-hidden">
			<div class="flex-shrink truncate pr-1 font-medium text-white">
				{mod.name.replace(/_/g, ' ')}
			</div>
			{#if mod.isPinned}
				<Icon class="flex-shrink-0 text-slate-400" icon="mdi:pin" />
			{/if}
			{#if mod.isDeprecated}
				<Icon class="flex-shrink-0 text-red-500" icon="mdi:error" />
			{/if}
			{#if mod.isInstalled}
				<Icon class="flex-shrink-0 text-green-500" icon="mdi:check-circle" />
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
			class="ml-2 mr-0.5 mt-0.5 hidden rounded-lg bg-green-600 p-2.5 align-middle text-2xl text-white hover:bg-green-500 group-hover:inline"
			on:click={(evt) => {
				dispatch('install', mod);
				evt.stopPropagation();
			}}
		>
			<Icon icon="mdi:download" />
		</Button.Root>
	{/if}
</button>
