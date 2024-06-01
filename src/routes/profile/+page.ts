import type { PageLoad } from './$types';
import { invokeCommand } from '$lib/invoke';
import type { QueryModsArgs } from '$lib/models';

export const load: PageLoad = async () => {
	let queryArgs = await invokeCommand<QueryModsArgs>('get_query_args', { source: 'profile' });
  
  return {
    queryArgs
  };
};
