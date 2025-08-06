import * as api from '$lib/api';
import type { ProfileInfo, ManagedGameInfo } from '$lib/types';
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

	update = async (info: ManagedGameInfo) => {
		this.list = info.profiles;
		this.activeId = info.activeId;
	}

	updateOne = async (info: ProfileInfo) => {
		const index = this.list.findIndex(profile => profile.id == info.id);
		if (index === -1) {
			this.list.push(info);
		} else {
			this.list[index] = info;
		}
	}

	refresh = async () => {
		const info = await api.profile.getInfo();
		profiles.update(info);
	}

	setActive = async (index: number) => {
		await api.profile.setActive(index);

		const sync = this.active?.sync;
		if (!sync) return;

		await api.profile.sync.fetch();
	};
}

const profiles = new ProfilesState();

profiles.refresh();

export default profiles;
