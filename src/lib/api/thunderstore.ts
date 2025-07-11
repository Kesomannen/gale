import { invoke } from '$lib/invoke';
import type { MarkdownCache, Mod, ModId, QueryModsArgs } from '$lib/types';

export const query = (args: QueryModsArgs) => invoke<Mod[]>('query_thunderstore', { args });
export const stopQuerying = () => invoke('stop_querying_thunderstore');
export const triggerModFetch = () => invoke('trigger_mod_fetch');
export const getMarkdown = (id: ModId, kind: MarkdownCache) =>
	invoke<string | null>('get_markdown', { modRef: id, kind });
export const setToken = (token: string) => invoke('set_thunderstore_token', { token });
export const hasToken = () => invoke<boolean>('has_thunderstore_token');
export const clearToken = () => invoke('clear_thunderstore_token');
