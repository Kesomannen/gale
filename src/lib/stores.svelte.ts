import { writable } from 'svelte/store';
import type { QueryModsArgsWithoutMax } from './types';
import games from '$lib/state/game.svelte';

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

$effect(() => {
	if (games.active === null) {
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
