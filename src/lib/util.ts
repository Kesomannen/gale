import { get } from 'svelte/store';
import type { Mod, ConfigEntry, Dependant, SyncUser } from './types';
import { activeGame } from './stores.svelte';
import { convertFileSrc } from '@tauri-apps/api/core';

export function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

export function formatTime(seconds: number): string {
	if (seconds < 60) {
		return `${Math.round(seconds)} seconds`;
	}

	if (seconds < 3600) {
		let minutes = Math.floor(seconds / 60);
		return `${minutes} minute${minutes > 1 ? 's' : ''}`;
	}

	let hours = Math.floor(seconds / 3600);
	return `${hours} hour${hours > 1 ? 's' : ''}`;
}

export function shortenNum(value: number): string {
	var i = value == 0 ? 0 : Math.floor(Math.log(value) / Math.log(1000));
	if (i === 0) {
		return value.toString();
	}
	return (value / Math.pow(1000, i)).toFixed(1) + ['', 'k', 'M', 'G', 'T'][i];
}

export function timeSince(date: Date | string): string {
	let seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);

	let [interval, str] = (() => {
		let interval = Math.floor(seconds / (60 * 60 * 24 * 365.25));
		if (interval >= 1) return [interval, 'year'];

		interval = Math.floor(seconds / (60 * 60 * 24 * 30));
		if (interval >= 1) return [interval, 'month'];

		interval = Math.floor(seconds / (60 * 60 * 24 * 7));
		if (interval >= 1) return [interval, 'week'];

		interval = Math.floor(seconds / (60 * 60 * 24));
		if (interval >= 1) return [interval, 'day'];

		interval = Math.floor(seconds / (60 * 60));
		if (interval >= 1) return [interval, 'hour'];

		interval = Math.floor(seconds / 60);
		if (interval >= 1) return [interval, 'second'];

		return [null, null];
	})();

	switch (interval) {
		case null:
			return 'a moment';
		case 1:
			return `a ${str}`;
		default:
			return `${interval} ${str}s`;
	}
}

export function isOutdated(mod: Mod): boolean {
	if (mod.versions.length === 0) {
		return false;
	}

	return mod.version !== mod.versions[0].name;
}

export function communityUrl(path: string) {
	return `https://thunderstore.io/c/${get(activeGame)?.slug}/p/${path}/`;
}

export function iconSrc(mod: Mod) {
	if (mod.type === 'remote') {
		let fullName = `${mod.author}-${mod.name}-${mod.version}`;
		return thunderstoreIconUrl(fullName);
	} else if (mod.icon !== null) {
		return convertFileSrc(mod.icon);
	} else {
		return `games/${get(activeGame)?.slug}.webp`;
	}
}

export function thunderstoreIconUrl(fullName: string) {
	return `https://gcdn.thunderstore.io/live/repository/icons/${fullName}.png`;
}

export function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1);
}

export interface ListSeparator {
	type: 'default' | 'custom';
	char: string;
}

const listSeparatorKeyword = 'ListSeparator=';

export function getListSeparator({ description }: ConfigEntry): ListSeparator {
	if (description !== null) {
		let separatorIndex = description.indexOf(listSeparatorKeyword);

		if (separatorIndex !== -1) {
			return { type: 'custom', char: description[separatorIndex + listSeparatorKeyword.length] };
		}
	}

	return { type: 'default', char: ',' };
}

export function fileToBase64(file: File): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.readAsDataURL(file);
		reader.onload = () => {
			const result = reader.result as string;
			resolve(result.split(',')[1]); // Extract only the Base64 part
		};
		reader.onerror = (error) => reject(error);
	});
}

export function isValidHex(str: string) {
	return /^([0-9A-Fa-f]{6})$/.test(str);
}

export function discordAvatarUrl(user: SyncUser) {
	return `https://cdn.discordapp.com/avatars/${user.discordId}/${user.avatar}.png`;
}

export function selectItems(
	items: string[],
	getLabel: (item: string) => string = (value) => value as string
) {
	return items.map((item) => ({ value: item, label: getLabel(item) }));
}

export function emptyOrUndefined(str: string) {
	if (str.length === 0) return undefined;
	return str;
}
