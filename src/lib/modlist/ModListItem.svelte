<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';
	import { isOutdated } from '$lib/util';

	const FALLBACK_ICON =
		'https://sm.ign.com/t/ign_es/cover/l/lethal-com/lethal-company_817h.300.jpg';

	export let mod: Mod;
	export let isSelected: boolean;
	export let onClick: (mod: Mod) => void;
</script>

<Button.Root
	class="flex border border-slate-500 rounded-lg p-2 group my-0.5 w-full {isSelected
		? 'bg-slate-700'
		: 'hover:bg-slate-700 border-opacity-0'}"
	on:click={() => onClick(mod)}
>
	<img src={mod.icon ?? FALLBACK_ICON} alt={mod.name} class="w-12 h-12 rounded-md" />
	<div class="pl-3 overflow-hidden flex-grow flex-shrink align-middle text-left">
		<span
			class="font-medium {mod.enabled === false ? 'line-through text-slate-300' : 'text-white'}"
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
		<slot />
	</div>
</Button.Root>
