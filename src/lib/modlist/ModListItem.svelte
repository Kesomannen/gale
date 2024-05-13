<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';

	const FALLBACK_ICON = 'https://sm.ign.com/t/ign_es/cover/l/lethal-com/lethal-company_817h.300.jpg';

	export let mod: Mod;
	export let isSelected: boolean;
	export let onClick: (mod: Mod) => void;
</script>

<Button.Root
	class="flex border border-slate-500 {isSelected ? 'bg-slate-700' : 'hover:bg-slate-700 border-opacity-0'} rounded-lg p-2 items-center group"
	on:click={() => onClick(mod)}
>
	<img src={mod.icon ?? FALLBACK_ICON} alt="Mod icon" class="w-12 h-12 rounded-md group-hover:shadow-xl" />
	<div class="pl-4 overflow-hidden flex-grow flex-shrink">
		<div class="flex items-center">
			<div class="text-slate-100 group-hover:text-white font-medium {mod.enabled === false && 'line-through'}">
				{mod.name}
			</div>
			<div class="text-slate-400 font-light pl-2 pr-1 {mod.enabled === false && 'line-through'}">
				{mod.version ?? ""}
			</div>
			{#if mod.isPinned}
				<Icon class="ml-1 text-slate-500" icon="mdi:pin" />
			{/if}
			{#if mod.enabled === false}
				<Icon class="ml-1 text-yellow-400" icon="mdi:eye-off" />
			{/if}
			{#if mod.isDeprecated}
				<Icon class="ml-1 text-red-500" icon="mdi:error" />
			{/if}
			{#if isOutdated(mod)}
				<Icon class="ml-1 text-blue-500" icon="mdi:arrow-up-circle" />
			{/if}
		</div>
		<div class="text-slate-300 group-hover:text-slate-200 truncate text-left {mod.enabled === false && 'line-through'}">
			{mod.description ?? ""}
		</div>
	</div>

	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<div class="contents" on:click={evt => evt.stopPropagation()} role="none">
		<slot />
	</div>
</Button.Root>
