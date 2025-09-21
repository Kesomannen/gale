import { invoke } from '$lib/invoke';
import type { LaunchOption } from '$lib/types';

export const launchGame = (args?: string) => invoke('launch_game', { args });
export const getArgs = () => invoke<string>('get_launch_args');
export const openGameDir = () => invoke('open_game_dir');
export const getSteamLaunchOptions = () => invoke<LaunchOption[]>('get_steam_launch_options');
