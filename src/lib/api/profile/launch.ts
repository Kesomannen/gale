import { invoke } from '$lib/invoke';

export const launchGame = () => invoke('launch_game');
export const getArgs = () => invoke<string>('get_launch_args');
export const openGameDir = () => invoke('open_game_dir');
