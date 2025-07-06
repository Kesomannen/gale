import * as api from '$lib/api';
import type { SyncUser } from '$lib/types';

class UserState {
	value: SyncUser | null = $state(null);

	refresh = async () => {
		this.value = await api.profile.sync.getUser();
	};

	login = async () => {
		this.value = await api.profile.sync.login();
	};

	logout = async () => {
		await api.profile.sync.logout();
		this.value = null;
	};
}

const user = new UserState();

user.refresh();

console.log('user.svelte.ts');

export default user;
