import * as api from '$lib/api';
import { derived, get, writable } from 'svelte/store';
import type {
	FiltersResponse,
	Game,
	GameInfo,
	PackageCategory,
	ProfileInfo,
	ProfilesInfo,
	QueryModsArgs,
	QueryModsArgsWithoutMax,
	SyncUser
} from './types';
import { fetch } from '@tauri-apps/plugin-http';

export let games: { list: Game[] } = $state({
	list: []
});

export let categories = writable<PackageCategory[]>([]);
export let activeGame = writable<Game | null>(null);

export let activeProfileId: number = 0;
export let profiles = writable<ProfileInfo[]>([]);
export let activeProfile = writable<ProfileInfo | null>(null);

export let user = writable<SyncUser | null>(null);

export let activeProfileLocked = derived([activeProfile, user], ([activeProfile, user]) => {
	if (activeProfile === null) return false;
	if (activeProfile.sync === null) return false;
	if (user === null) return true;

	return activeProfile.sync.owner.discordId != user.discordId;
});

const defaultModQuery: () => QueryModsArgsWithoutMax = () => ({
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

const defaultProfileQuery: () => QueryModsArgsWithoutMax = () => ({
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

function loadQuery(key: string, getDefault: () => QueryModsArgsWithoutMax) {
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

function createQueryStore(key: string, getDefault: () => QueryModsArgsWithoutMax) {
	let store = writable<QueryModsArgsWithoutMax>(loadQuery(key, getDefault));
	store.subscribe((value) => {
		localStorage.setItem(key, JSON.stringify(value));
	});
	return store;
}

export async function refreshGames() {
	console.log('refreshing games...');

	const info: GameInfo = await api.profile.getGameInfo();

	for (let game of info.all) {
		game.favorite = info.favorites.includes(game.slug);
	}

	games.list = info.all;

	activeGame.set(info.active);
	refreshCategories();
	refreshProfiles();
}

export async function setActiveGame(slug: string) {
	await api.profile.setActiveGame(slug);
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
	let info = await api.profile.getInfo();

	activeProfileId = info.activeId;
	profiles.set(info.profiles);
	activeProfile.set(info.profiles.find((profile) => profile.id === activeProfileId) ?? null);
}

export async function setActiveProfile(index: number) {
	await api.profile.setActive(index);
	await refreshProfiles();

	const sync = get(activeProfile)?.sync;
	if (!sync) return;

	await api.profile.sync.fetch();
	await refreshProfiles();
}

export async function refreshUser() {
	let info = await api.profile.sync.getUser();
	user.set(info);
}

export async function login() {
	let info = await api.profile.sync.login();
	user.set(info);
}

export async function logout() {
	await api.profile.sync.logout();
	user.set(null);
}
