import { invoke } from '$lib/invoke';
import type { GameInfo, ProfileInfo } from '$lib/models';
import { listen } from '@tauri-apps/api/event';

class Games {
	all: GameInfo[] = $state([]);
	active: GameInfo | undefined = $state();

	setActive(id: number) {
		this.active = getGame(id);
		setLocalStorageInt('activeGame', games.active?.id ?? 1);
	}
}

const games = new Games();

class Profiles {
	active: ProfileInfo | undefined = $state();

	get activeId() {
		if (this.active !== undefined) {
			return this.active.id;
		}

		console.warn('no active profile');
		return 1;
	}

	async setActive(id: number) {
		this.active = await invoke('profile', 'get', { id });
		setLocalStorageInt('activeProfile', profiles.active?.id ?? 1);
	}
}

const profiles = new Profiles();

listen<ProfileInfo>('profile-update', async ({ payload }) => {
	if (profiles.active?.id === payload.id) {
		profiles.active = payload;
	}
})

fetchGames();

let activeProfileId = getLocalStorageInt('activeProfile', 1);
profiles.setActive(activeProfileId);

function getGame(id: number): GameInfo {
	return games.all.find((community) => community.id === id)!;
}

function getLocalStorageInt(key: string, def: number): number {
	let value = localStorage.getItem(key);
	if (value === null) {
		return def;
	}
	return parseInt(value);
}

function setLocalStorageInt(key: string, value: number) {
	localStorage.setItem(key, value.toString());
}

async function fetchGames() {
	let id = getLocalStorageInt('activeGame', 1);
	games.all = await invoke('core', 'get_games');
	games.active = getGame(id);
}

export { games, profiles };
