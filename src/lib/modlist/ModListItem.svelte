<script lang="ts">
	import { Button } from 'bits-ui';
	import type { Mod } from '../models';
	import Icon from '@iconify/svelte';

	const FALLBACK_ICON = 'https://sm.ign.com/t/ign_es/cover/l/lethal-com/lethal-company_817h.300.jpg';

	export let mod: Mod;
	export let onClick: (mod: Mod) => void;
</script>

<Button.Root
	class="flex hover:bg-gray-700 rounded-xl p-2 items-center group"
	on:click={() => onClick(mod)}
>
	<img src={mod.icon ?? FALLBACK_ICON} alt="Mod icon" class="w-12 h-12 rounded-lg group-hover:shadow-xl" />
	<div class="pl-4 overflow-hidden flex-grow">
		<div class="flex flex-row">
			<div class="text-slate-100 group-hover:text-white font-semibold">
				{mod.name}
			</div>
			<div class="text-slate-500 group-hover:text-slate-400 font-light pl-2">
				{mod.version ?? ""}
			</div>
			{#if mod.isPinned}
				<Icon class="ml-2 mt-1 text-slate-400" icon="mdi:pin" />
			{/if}
		</div>
		<div class="text-slate-300 group-hover:text-slate-200 truncate text-left">
			{mod.description ?? ""}
		</div>
	</div>
</Button.Root>
