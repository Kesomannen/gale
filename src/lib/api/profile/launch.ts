import { invoke } from '$lib/invoke';

export const launchGame = (vanilla: boolean) => invoke('launch_game', { vanilla });
export const getArgs = () => invoke<string>('get_launch_args');
export const openGameDir = () => invoke('open_game_dir');
