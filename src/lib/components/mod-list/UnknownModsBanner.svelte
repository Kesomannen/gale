<script lang="ts">
	import type { Dependant } from '$lib/types';
	import Icon from '@iconify/svelte';

	type Props = {
		mods: Dependant[];
		uninstall: (mod: Dependant) => void;
	};

	let { mods, uninstall }: Props = $props();
</script>

<div class="mr-3 mb-1 flex items-center rounded-lg bg-red-600 py-1.5 pr-1 pl-3 text-red-100">
	<Icon icon="mdi:alert-circle" class="mr-2 text-xl" />
	The following {mods.length === 1 ? 'mod' : 'mods'} could not be found: {mods
		.map((mod) => mod.fullName)
		.join(', ')}.
	<button
		class="ml-1 font-semibold text-white hover:text-red-100 hover:underline"
		onclick={() => {
			mods.forEach(uninstall);
		}}
	>
		Uninstall {mods.length === 1 ? 'it' : 'them'}?
	</button>
</div>
