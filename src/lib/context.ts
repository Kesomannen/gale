import { writable } from 'svelte/store';
import type { ModContextItem } from './types';

function openIfNotNull(url: string | null) {
	if (url !== null) open(url);
}

export const defaultContextItems: ModContextItem[] = [
	{
		label: 'Open website',
		icon: 'mdi:open-in-new',
		onclick: (mod) => openIfNotNull(mod.websiteUrl),
		showFor: (mod) => mod.websiteUrl !== null && mod.websiteUrl.length > 0
	},
	{
		label: 'Donate',
		icon: 'mdi:heart',
		onclick: (mod) => openIfNotNull(mod.donateUrl),
		showFor: (mod) => mod.donateUrl !== null
	}
];

export let activeContextMenu = writable<string | null>(null);
