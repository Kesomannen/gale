import { writable } from 'svelte/store';
import { invokeCommand } from './invoke';

export let activeProfileIndex: number = 0;
export let profileNames: string[] = [];

export const currentProfile = writable<string>('Loading...');
export const inProfile = writable<boolean>(false);

refreshProfiles();

interface ProfileInfo {
	active_index: number;
	names: string[];
}

export async function refreshProfiles() {
	const info: ProfileInfo = await invokeCommand('get_profile_info');
	activeProfileIndex = info.active_index;
	profileNames = info.names;
	currentProfile.set(profileNames[activeProfileIndex]);
}

export async function setActiveProfile(index: number) {
	await invokeCommand('set_active_profile', { index });
	refreshProfiles();
}
