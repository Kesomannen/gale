import { get, writable } from 'svelte/store';
import { invokeCommand } from './invoke';
import { SortBy, SortOrder, type FiltersResponse, type Game, type GameInfo, type PackageCategory, type ProfileInfo, type ProfilesInfo, type QueryModsArgs } from './models';
import { fetch } from '@tauri-apps/api/http';

export let games: Game[] = [];
export let categories = writable<PackageCategory[]>([]);
export const currentGame = writable<Game | undefined>(undefined);

export let activeProfileIndex: number = 0;
export let profiles: ProfileInfo[] = [];
export const currentProfile = writable<ProfileInfo>({
	name: '',
	modCount: 0
});

refreshGames();

const defaultModQuery: () => QueryModsArgs = () => ({
	maxCount: 20,
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: false,
	includeDeprecated: false,
	includeDisabled: false,
	sortBy: SortBy.LastUpdated,
	sortOrder: SortOrder.Descending
})

export let modQuery = writable(defaultModQuery());

currentGame.subscribe(() => {
	modQuery.set(defaultModQuery());
});

export let profileQuery = writable({
	maxCount: 20,
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: true,
	includeDeprecated: true,
	includeDisabled: true,
	sortBy: SortBy.Custom,
	sortOrder: SortOrder.Descending
});

export async function refreshGames() {
	const info: GameInfo = await invokeCommand('get_game_info');
	games = info.all;

	for (let game of games) {
		game.favorite = info.favorites.includes(game.id);
	}

	currentGame.set(info.active);
	refreshProfiles();
	refreshCategories();
}

export async function setActiveGame(game: Game) {
	await invokeCommand('set_active_game', { id: game.id });
	refreshGames();
}

export async function refreshCategories() {
	let gameId = get(currentGame)?.id;
	if (!gameId) return;

	let response = await fetch<FiltersResponse>(`https://thunderstore.io/api/cyberstorm/community/${gameId}/filters/`);
	categories.set(response.data.package_categories);
}

export async function refreshProfiles() {
	let info = await invokeCommand<ProfilesInfo>('get_profile_info');

	activeProfileIndex = info.activeIndex;
	profiles = info.profiles;
	currentProfile.set(profiles[activeProfileIndex]);
}

export async function setActiveProfile(index: number) {
	await invokeCommand('set_active_profile', { index });
	refreshProfiles();
}
