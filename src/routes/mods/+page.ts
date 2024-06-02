import { invokeCommand } from '$lib/invoke';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';
import { modQuery } from '$lib/stores';
import type { Mod } from '$lib/models';

export const load: PageLoad = async () => {
	let args = get(modQuery);

	let mods = await invokeCommand<Mod[]>('query_thunderstore', { args });

	return {
		mods
	};
};
