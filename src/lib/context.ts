import { writable } from 'svelte/store';
import { ModType, type ModContextItem } from './types';
import { open } from '@tauri-apps/plugin-shell';
import { m } from './paraglide/messages';
import { communityUrl } from './util';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { pushInfoToast } from './toast';

function openIfNotNull(url: string | null) {
	if (url !== null) open(url);
}

export const defaultContextItems: ModContextItem[] = [
	{
		label: m.page_modContextItem_openWebsite(),
		icon: 'mdi:open-in-new',
		onclick: (mod) => openIfNotNull(mod.websiteUrl),
		showFor: (mod) => mod.websiteUrl !== null && mod.websiteUrl.length > 0
	},
	{
		label: m.page_modContextItem_copyLink(),
		icon: 'mdi:link-variant',
		onclick: async (mod) => {
			const url = communityUrl(mod.backend, mod.author ?? '', mod.name);
			await writeText(url);
			pushInfoToast({
				message: m.page_modContextItem_copyLink_message()
			});
		},
		showFor: (mod) => mod.type === ModType.Remote
	},
	{
		label: m.page_modContextItem_donate(),
		icon: 'mdi:heart',
		onclick: (mod) => openIfNotNull(mod.donateUrl),
		showFor: (mod) => mod.donateUrl !== null
	}
];

export let activeContextMenu = writable<string | null>(null);
