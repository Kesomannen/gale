import { invoke } from '$lib/invoke';
import { type Backend, type MarkdownType, type Mod, type ModId, type QueryModsArgs } from '$lib/types';

export const query = (args: QueryModsArgs) => invoke<Mod[]>('query_thunderstore', { args });
export const stopQuerying = () => invoke('stop_querying_thunderstore');
export const triggerModFetch = () => invoke('trigger_mod_fetch');
export const getMarkdown = (id: ModId, type: MarkdownType) =>
	invoke<string | null>('get_markdown', { modRef: id, kind: type });
export const setToken = (backend: Backend, token: string) => invoke('set_api_token', { backend, token });
export const hasToken = (backend: Backend) => invoke<boolean>('has_api_token', { backend });
export const clearToken = (backend: Backend) => invoke('clear_api_token', { backend });
