import type { FiltersResponse, Game, GameInfo, PackageCategory } from '$lib/types';
import * as api from '$lib/api';
import { pushToast } from '$lib/toast';
import { fetch } from '@tauri-apps/plugin-http';
import profiles from './profile.svelte';

class GamesState {
	active: Game | null = $state(null);
	list: Game[] = $state([]);
	categories: PackageCategory[] = $state([]);

	refresh = async () => {
		const info: GameInfo = await api.profile.getGameInfo();

		for (let game of info.all) {
			game.favorite = info.favorites.includes(game.slug);
		}

		this.active = info.active;
		this.list = info.all;

		this.#refreshCategories();
		profiles.refresh();
	};

	#refreshCategories = async () => {
		const slug = this.active?.slug;
		if (!slug) return;

		try {
			const url = `https://thunderstore.io/api/experimental/community/${slug}/category/`;
			const response = await fetch(url);

			if (!response.ok) {
				throw new Error(await response.text());
			}

			const data = (await response.json()) as FiltersResponse;
			this.categories = data.results.sort((a, b) => a.name.localeCompare(b.name));
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Failed to fetch categories',
				message: JSON.stringify(err)
			});
		}
	};

	setActive = async (slug: string) => {
		await api.profile.setActiveGame(slug);
		await this.refresh();
	};
}

const games = new GamesState();

games.refresh();

export default games;
