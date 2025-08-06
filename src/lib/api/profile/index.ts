import { invoke } from '$lib/invoke';
import type {
	Game,
	GameInfo,
	ModActionResponse,
	ProfileQuery,
	ManagedGameInfo,
	QueryModsArgs
} from '$lib/types';

export * as export from './export';
export * as import from './import';
export * as install from './install';
export * as launch from './launch';
export * as sync from './sync';
export * as update from './update';

export const getGameInfo = () => invoke<GameInfo>('get_game_info');
export const favoriteGame = (slug: string) => invoke('favorite_game', { slug });
export const setActiveGame = (slug: string) => invoke('set_active_game', { slug });
export const getInfo = () => invoke<ManagedGameInfo>('get_profile_info');
export const setActive = (index: number) => invoke('set_active_profile', { index });
export const query = (args: QueryModsArgs) => invoke<ProfileQuery>('query_profile', { args });
export const isModInstalled = (uuid: string) => invoke<boolean>('is_mod_installed', { uuid });
export const create = (name: string, overridePath: string | null) =>
	invoke('create_profile', { name, overridePath });
export const deleteProfile = (index: number) => invoke('delete_profile', { index });
export const rename = (name: string) => invoke('rename_profile', { name });
export const duplicate = (name: string) => invoke('duplicate_profile', { name });
export const removeMod = (uuid: string) => invoke<ModActionResponse>('remove_mod', { uuid });
export const toggleMod = (uuid: string) => invoke<ModActionResponse>('toggle_mod', { uuid });
export const forceRemoveMods = (uuids: string[]) => invoke('force_remove_mods', { uuids });
export const forceToggleMods = (uuids: string[]) => invoke('force_toggle_mods', { uuids });
export const setAllModsState = (enable: boolean) =>
	invoke<number>('set_all_mods_state', { enable });
export const removeDisabledMods = () => invoke<number>('remove_disabled_mods');
export const getDependants = (uuid: string) => invoke<string[]>('get_dependants', { uuid });
export const openDir = () => invoke('open_profile_dir');
export const openModDir = (uuid: string) => invoke('open_mod_dir', { uuid });
export const openGameLog = () => invoke('open_game_log');
export const createDesktopShortcut = () => invoke('create_desktop_shortcut');
