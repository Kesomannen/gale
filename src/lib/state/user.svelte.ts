import * as api from '$lib/api';
import type { SyncUser } from '$lib/types';

class UserState {
	value: SyncUser | null = $state(null);

	refresh = async () => {
		this.value = await api.profile.sync.getUser();
	};

	login = async () => {
		const user = await api.profile.sync.login();
		this.value = user;
		return user;
	};

	logout = async () => {
		await api.profile.sync.logout();
		this.value = null;
	};
}

const user = new UserState();

user.refresh();

export default user;
