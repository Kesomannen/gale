import { derived, get, writable } from 'svelte/store';
import { invokeCommand } from './invoke';
import {
	SortBy,
	SortOrder,
	type FiltersResponse,
	type Game,
	type GameInfo,
	type PackageCategory,
	type ProfileInfo,
	type ProfilesInfo,
	type QueryModsArgs,
	type SyncUser
} from './models';
import { fetch } from '@tauri-apps/plugin-http';

export let games: Game[] = [];
export let categories = writable<PackageCategory[]>([]);
export let activeGame = writable<Game | null>(null);

export let activeProfileId: number = 0;
export let profiles: ProfileInfo[] = [];
export let activeProfile = writable<ProfileInfo | null>(null);

export let user = writable<SyncUser | null>(null);

export let activeProfileLocked = derived([activeProfile, user], ([activeProfile, user]) => {
	if (activeProfile === null) return false;
	if (activeProfile.sync === null) return false;
	if (user === null) return true;

	return activeProfile.sync.owner.id != user.id;
});

const defaultModQuery = () => ({
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: false,
	includeDeprecated: false,
	includeEnabled: false,
	includeDisabled: false,
	sortBy: SortBy.LastUpdated,
	sortOrder: SortOrder.Descending
});

const defaultProfileQuery = () => ({
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: true,
	includeDeprecated: true,
	includeEnabled: true,
	includeDisabled: true,
	sortBy: SortBy.Custom,
	sortOrder: SortOrder.Descending
});

export let modQuery = createQueryStore('modQuery', defaultModQuery);
export let profileQuery = createQueryStore('profileQuery', defaultProfileQuery);

let isFirst = true;
activeGame.subscribe((value) => {
	if (value === null) {
		return;
	}

	// ignore when the game is fetched on startup
	if (isFirst) {
		isFirst = false;
		return;
	}

	modQuery.set(defaultModQuery());
	profileQuery.set(defaultProfileQuery());
});

refreshGames();
refreshUser();

function loadQuery(key: string, getDefault: () => QueryModsArgs) {
	let json = localStorage.getItem(key);
	if (json) {
		try {
			let res = JSON.parse(json);
			// didn't have this field before
			res.includeEnabled = res.includeEnabled ?? true;
			return res;
		} catch (e) {
			console.error('Failed to parse stored query:', e);
		}
	}

	return getDefault();
}

function createQueryStore(key: string, getDefault: () => QueryModsArgs) {
	let store = writable<QueryModsArgs>(loadQuery(key, getDefault));
	store.subscribe((value) => {
		localStorage.setItem(key, JSON.stringify(value));
	});
	return store;
}

export async function refreshGames() {
	const info: GameInfo = await invokeCommand('get_game_info');
	games = info.all;

	for (let game of games) {
		game.favorite = info.favorites.includes(game.slug);
	}

	activeGame.set(info.active);
	refreshCategories();
	refreshProfiles();
}

export async function setActiveGame(slug: string) {
	await invokeCommand('set_active_game', { slug });
	await refreshGames();
}

export async function refreshCategories() {
	let gameId = get(activeGame)?.slug;
	if (!gameId) return;

	try {
		let response = await fetch(
			`https://thunderstore.io/api/experimental/community/${gameId}/category/`
		);

		if (!response.ok) {
			console.error('Failed to fetch categories:', response);
			return;
		}

		let data = (await response.json()) as FiltersResponse;
		categories.set(data.results);
	} catch (e) {
		console.error('Failed to fetch categories:', e);
	}
}

export async function refreshProfiles() {
	let info = await invokeCommand<ProfilesInfo>('get_profile_info');

	activeProfileId = info.activeId;
	profiles = info.profiles;
	activeProfile.set(profiles.find((profile) => profile.id === activeProfileId) ?? null);
}

export async function setActiveProfile(index: number) {
	await invokeCommand('set_active_profile', { index });
	await refreshProfiles();
}

export async function refreshUser() {
	let info = await invokeCommand<SyncUser | null>('get_user');
	user.set(info);
}

export async function login() {
	let info = await invokeCommand<SyncUser>('login');
	user.set(info);
}

export async function logout() {
	await invokeCommand('logout');
	user.set(null);
}
