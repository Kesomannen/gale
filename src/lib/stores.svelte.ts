import { derived, get, writable } from 'svelte/store';
import { invoke } from './invoke';
import type {
	FiltersResponse,
	Game,
	GameInfo,
	PackageCategory,
	ProfileInfo,
	ProfilesInfo,
	QueryModsArgs,
	SyncUser
} from './types';
import { fetch } from '@tauri-apps/plugin-http';

export let games: { list: Game[] } = $state({
	list: []
});

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

	return activeProfile.sync.owner.discordId != user.discordId;
});

const defaultModQuery: () => QueryModsArgs = () => ({
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: false,
	includeDeprecated: false,
	includeEnabled: false,
	includeDisabled: false,
	sortBy: 'rating',
	sortOrder: 'descending'
});

const defaultProfileQuery: () => QueryModsArgs = () => ({
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: true,
	includeDeprecated: true,
	includeEnabled: true,
	includeDisabled: true,
	sortBy: 'custom',
	sortOrder: 'descending'
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
	console.log('refreshing games...');

	const info: GameInfo = await invoke('get_game_info');
	
	for (let game of info.all) {
		game.favorite = info.favorites.includes(game.slug);
	}

	games.list = info.all;

	activeGame.set(info.active);
	refreshCategories();
	refreshProfiles();
}

export async function setActiveGame(slug: string) {
	await invoke('set_active_game', { slug });
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
	let info = await invoke<ProfilesInfo>('get_profile_info');

	activeProfileId = info.activeId;
	profiles = info.profiles;
	activeProfile.set(profiles.find((profile) => profile.id === activeProfileId) ?? null);
}

export async function setActiveProfile(index: number) {
	await invoke('set_active_profile', { index });
	await refreshProfiles();

	const sync = get(activeProfile)?.sync;
	if (!sync) return;

	await invoke('fetch_sync_profile');
	await refreshProfiles();
}

export async function refreshUser() {
	let info = await invoke<SyncUser | null>('get_user');
	user.set(info);
}

export async function login() {
	let info = await invoke<SyncUser>('login');
	user.set(info);
}

export async function logout() {
	await invoke('logout');
	user.set(null);
}
