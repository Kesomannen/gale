import { get } from 'svelte/store';
import type { Mod, ConfigEntry, Dependant } from './models';
import { activeGame } from './stores';
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

export function sentenceCase(str: string): string {
	const textcase = String(str)
		.replace(/^[^A-Za-z0-9]*|[^A-Za-z0-9]*$/g, '')
		.replace(/([a-z])([A-Z])/g, (m, a, b) => `${a}_${b.toLowerCase()}`)
		.replace(/[^A-Za-z0-9]+|_+/g, ' ')
		.toLowerCase();

	return textcase.charAt(0).toUpperCase() + textcase.slice(1);
}

export function titleCase(str: string): string {
	return String(str)
		.replace(/^[^A-Za-z0-9]*|[^A-Za-z0-9]*$/g, '')
		.replace(/([a-z])([A-Z])/g, (m, a, b) => `${a} ${b}`)
		.replace(/[^A-Za-z0-9]+|_+/g, ' ')
		.split(' ')
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
		.join(' ');
}

export function timeSince(date: Date): string {
	var seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
	var interval = Math.floor(seconds / 31536000);

	if (interval > 1) {
		return interval + ' years';
	}
	interval = Math.floor(seconds / 2592000);
	if (interval > 1) {
		return interval + ' months';
	}
	interval = Math.floor(seconds / 86400);
	if (interval > 1) {
		return interval + ' days';
	}
	interval = Math.floor(seconds / 3600);
	if (interval > 1) {
		return interval + ' hours';
	}
	interval = Math.floor(seconds / 60);
	if (interval > 1) {
		return interval + ' minutes';
	}
	return Math.floor(seconds) + ' seconds';
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
