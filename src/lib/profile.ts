import { writable } from 'svelte/store';
import { invokeCommand } from './invoke';
import type { Game, GameInfo, ProfileInfo } from './models';

export let games: Game[] = [];
export const currentGame = writable<Game | undefined>(undefined);

export let activeProfileIndex: number = 0;
export let profileNames: string[] = [];
export const currentProfile = writable<string>('Loading...');

refreshGames();

export async function refreshGames() {
	const info: GameInfo = await invokeCommand('get_game_info');
	games = info.all;

	for (let game of games) {
		game.favorite = info.favorites.includes(game.id);
	}

	currentGame.set(info.active);
	refreshProfiles();
}

export async function setActiveGame(game: Game) {
	await invokeCommand('set_active_game', { id: game.id });
	refreshGames();
}

export async function refreshProfiles() {
	const info: ProfileInfo = await invokeCommand('get_profile_info');
	activeProfileIndex = info.activeIndex;
	profileNames = info.names;
	currentProfile.set(profileNames[activeProfileIndex]);
}

export async function setActiveProfile(index: number) {
	await invokeCommand('set_active_profile', { index });
	refreshProfiles();
}
