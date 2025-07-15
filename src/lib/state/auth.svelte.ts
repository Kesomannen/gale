import * as api from '$lib/api';
import type { SyncUser } from '$lib/types';

class AuthState {
	user: SyncUser | null = $state(null);

	refresh = async () => {
		this.user = await api.profile.sync.getUser();
	};

	login = async () => {
		const user = await api.profile.sync.login();
		this.user = user;
		return user;
	};

	logout = async () => {
		await api.profile.sync.logout();
		this.user = null;
	};
}

const auth = new AuthState();

auth.refresh();

export default auth;
