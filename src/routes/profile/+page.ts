import { invokeCommand } from '$lib/invoke';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';
import { profileQuery } from '$lib/stores';
import type { ProfileQuery } from '$lib/models';

export const load: PageLoad = async () => {
	let args = get(profileQuery);

	let { mods, updates } = await invokeCommand<ProfileQuery>('query_profile', { args });

	return {
		mods,
		updates
	};
};
