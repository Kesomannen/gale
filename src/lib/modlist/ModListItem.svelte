<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';

	export let mod: Mod;
	export let onClick: (mod: Mod) => void;

	let isHovered = false;
</script>

<Button.Root
	class="flex hover:bg-gray-700 rounded-xl p-2 pl-r items-center group"
	on:click={() => onClick(mod)}
	on:mouseenter={() => (isHovered = true)}
	on:mouseleave={() => (isHovered = false)}
>
	<img src={mod.version.icon} alt="Mod icon" class="w-12 h-12 rounded-lg" />
	<div class="pl-4 overflow-hidden flex-grow">
		<div class="flex flex-row">
			<div class="text-slate-100 group-hover:text-white font-semibold">
				{mod.version.name}
			</div>
			<div class="text-slate-500 group-hover:text-slate-400 font-light pl-2">
				{mod.version.version_number}
			</div>
			{#if mod.package.is_pinned}
				<Icon class="ml-2 mt-1 text-slate-400" icon="mdi:pin" />
			{/if}
		</div>
		<div class="text-slate-300 group-hover:text-slate-200 truncate text-left">
			{mod.version.description}
		</div>
	</div>
	{#if isHovered}
		<slot />
	{/if}
</Button.Root>
