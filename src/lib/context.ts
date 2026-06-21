import { writable } from 'svelte/store';
import type { ModContextItem } from './types';
import { open } from '@tauri-apps/plugin-shell';
import { m } from './paraglide/messages';

function openIfNotNull(url: string | null) {
	if (url !== null) open(url);
}

export const defaultContextItems: ModContextItem[] = [
	{
		label: m.page_modContextItem_openWebsite(),
		icon: 'ph:arrow-square-out-fill',
		onclick: (mod) => openIfNotNull(mod.websiteUrl),
		showFor: (mod) => mod.websiteUrl !== null && mod.websiteUrl.length > 0
	},
	{
		label: m.page_modContextItem_donate(),
		icon: 'ph:heart-fill',
		onclick: (mod) => openIfNotNull(mod.donateUrl),
		showFor: (mod) => mod.donateUrl !== null
	}
];

export let activeContextMenu = writable<string | null>(null);
