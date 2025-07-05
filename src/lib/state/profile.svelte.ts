import * as api from '$lib/api';
import type { ProfileInfo } from '$lib/types';

class ProfileState {
	list: ProfileInfo[] = $state([]);
	active: ProfileInfo | null = $state(null);
}

const profile = new ProfileState();
export default profile;
