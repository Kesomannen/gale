import { Backend, type Game, type PackageCategory } from '$lib/types';
import * as api from '$lib/api';

class GamesState {
	active: Game | null = $state(null);
	lastUpdated: string = $state('');
	list: Game[] = $state([]);
	categories: PackageCategory[] = $state([]);
	activeBackends: Backend[] = $derived(this.active?.backends ?? []);

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
		this.categories = await api.thunderstore.getCategories(slug);
	};

	setActive = async (slug: string) => {
		await api.profile.setActiveGame(slug);
		await this.refresh();
	};
}

const games = new GamesState();

games.refresh();

export default games;
