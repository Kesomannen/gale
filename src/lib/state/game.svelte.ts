import {
	Backend,
	Backends,
	type FiltersResponse,
	type Game,
	type PackageCategory
} from '$lib/types';
import * as api from '$lib/api';
import { pushToast } from '$lib/toast';
import { fetch } from '@tauri-apps/plugin-http';

class GamesState {
	active: Game | null = $state(null);
	lastUpdated: string = $state('');
	list: Game[] = $state([]);
	categories: PackageCategory[] = $state([]);

	refresh = async () => {
		const info = await api.profile.getGameInfo();

		for (let game of info.all) {
			game.favorite = info.favorites.includes(game.slug);
		}

		this.active = info.active;
		this.lastUpdated = info.lastUpdated;
		this.list = info.all;

		this.#refreshCategories();
	};

	#refreshCategories = async () => {
		const slug = this.active?.slug;
		if (!slug) return;

		async function fetchCategories(baseUrl: string) {
			try {
				const url = `${baseUrl}/experimental/community/${slug}/category/`;
				const response = await fetch(url);

				if (!response.ok) {
					const message = await response.text();
					throw new Error(`${response.status} ${response.statusText}: ${message}`);
				}

				return (await response.json()) as FiltersResponse;
			} catch (err) {
				pushToast({
					type: 'error',
					name: 'Failed to fetch categories',
					message: err instanceof Error ? err.message : String(err)
				});
				return { results: [] };
			}
		}

		let thunderstoreBackend = 'https://thunderstore.io/api';
		let hexiumBackend = 'https://hexium.gg/api';
		let backends: string[] = [];
		if ((this.active?.backends?.length || 0) > 1) {
			// for now assume this means both backends are available
			let prefs = await api.prefs.get();
			prefs.gamePrefs = new Map(Object.entries(prefs.gamePrefs));
			switch (prefs.gamePrefs.get(slug)?.backend || Backends.All) {
				case Backends.Thunderstore:
					backends = [thunderstoreBackend];
					break;
				case Backends.Hexium:
					backends = [hexiumBackend];
					break;
				case Backends.All:
					backends = [thunderstoreBackend, hexiumBackend];
			}
		} else if (this.active?.backends?.includes(Backend.Hexium)) {
			backends = [hexiumBackend];
		} else {
			backends = [thunderstoreBackend];
		}

		// Deduplicate categories from all sources
		Promise.allSettled(backends.map(fetchCategories)).then((results) => {
			this.categories = [
				...new Set(
					results.flatMap((result) => (result.status === 'fulfilled' ? result.value.results : []))
				)
			].sort((a, b) => a.name.localeCompare(b.name));
		});
	};

	setActive = async (slug: string) => {
		await api.profile.setActiveGame(slug);
		await this.refresh();
	};
}

const games = new GamesState();

games.refresh();

export default games;
