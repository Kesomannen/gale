import * as api from '$lib/api';
import type { ProfileInfo } from '$lib/types';
import user from './user.svelte';

class ProfilesState {
	list: ProfileInfo[] = $state([]);
	active: ProfileInfo | null = $state(null);

	activeId = $derived(this.active?.id ?? null);

	activeLocked = $derived.by(() => {
		if (this.active === null) return false;
		if (this.active.sync === null) return false;
		if (user.value === null) return true;

		return this.active.sync.owner.discordId != user.value.discordId;
	});

	refresh = async () => {
		const info = await api.profile.getInfo();

		this.list = info.profiles;
		this.active = info.profiles.find((profile) => profile.id === info.activeId) ?? null;
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
