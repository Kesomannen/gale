import * as api from '$lib/api';
import type { ProfileInfo } from '$lib/types';
import auth from './auth.svelte';

class ProfilesState {
	list: ProfileInfo[] = $state([]);
	activeId: number | null = $state(null);

	active: ProfileInfo | null = $derived(this.list.find(profile => profile.id === this.activeId) ?? null);

	activeLocked = $derived.by(() => {
		if (this.active === null) return false;
		if (this.active.sync === null) return false;
		if (auth.user === null) return true;

		return this.active.sync.owner.discordId != auth.user.discordId;
	});

	refresh = async () => {
		const info = await api.profile.getInfo();

		this.list = info.profiles;
		this.activeId = info.activeId;
	};

	setActive = async (index: number) => {
		await api.profile.setActive(index);
		await this.refresh();

		const sync = this.active?.sync;
		if (!sync) return;

		await api.profile.sync.fetch();
		await this.refresh();
	};
}

const profiles = new ProfilesState();
export default profiles;
