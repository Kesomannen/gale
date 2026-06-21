<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Dependant } from '$lib/types';
	import Icon from '@iconify/svelte';

	type Props = {
		mods: Dependant[];
		uninstall: (mod: Dependant) => void;
	};

	let { mods, uninstall }: Props = $props();
</script>

<div class="mr-3 mb-1 flex items-center rounded-lg bg-red-600 py-1.5 pr-1 pl-3 text-red-100">
	<Icon icon="ph:warning-circle-fill" class="mr-2 text-xl" />
	{m.unknownModsBanner_content({ count: mods.length })}
	{mods.map((mod) => mod.fullName).join(', ')}.
	<button
		class="ml-1 font-semibold text-white hover:text-red-100 hover:underline"
		onclick={() => {
			mods.forEach(uninstall);
		}}
	>
		{m[`unknownModsBanner_uninstall_${mods.length === 1 ? 'it' : 'them'}`]()}
	</button>
</div>
