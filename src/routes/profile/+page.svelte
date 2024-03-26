<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { Mod, ModQueryArgs } from '$lib/models';
	import ModList from '$lib/modlist/ModList.svelte';

	let mods: Mod[];
	let queryArgs: ModQueryArgs;

	$: {
		if (queryArgs) {
			invokeCommand<Mod[]>('query_mods_in_profile', { args: queryArgs })
				.then((result) => mods = result)
		}
	}
</script>

<ModList bind:mods={mods} bind:queryArgs={queryArgs} />