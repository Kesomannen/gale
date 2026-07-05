import {
	Backend,
	type ConfigEntry,
	type Game,
	type MarkdownType,
	type Mod,
	ModType
} from './types';
import { convertFileSrc } from '@tauri-apps/api/core';
import games from './state/game.svelte';
import { isLatinAlphabet } from './i18n';
import { m } from './paraglide/messages';
import * as api from '$lib/api';
import { getLocale } from './paraglide/runtime';

export function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

export function formatModName(name: string): string {
	return name.replace(/_/g, ' ');
}

export function shortenNum(value: number): string {
	var i = value == 0 ? 0 : Math.floor(Math.log(value) / Math.log(1000));
	if (i === 0) {
		return value.toString();
	}
	return (value / Math.pow(1000, i)).toFixed(1) + ['', 'k', 'M', 'G', 'T'][i];
}

const rtf = new Intl.RelativeTimeFormat(getLocale(), { numeric: 'auto' });

export function timeSince(date: Date | string): string {
	let seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);

	let value = Math.floor(seconds / (60 * 60 * 24 * 365.25));
	if (value > 0) return rtf.format(-value, 'year');

	value = Math.floor(seconds / (60 * 60 * 24 * 30));
	if (value > 0) return rtf.format(-value, 'month');

	value = Math.floor(seconds / (60 * 60 * 24 * 7));
	if (value > 0) return rtf.format(-value, 'week');

	value = Math.floor(seconds / (60 * 60 * 24));
	if (value > 0) return rtf.format(-value, 'day');

	value = Math.floor(seconds / (60 * 60));
	if (value > 0) return rtf.format(-value, 'hour');

	value = Math.floor(seconds / 60);
	if (value > 0) return rtf.format(-value, 'minute');

	return m.util_timeSince_interval_null();
}

export function isOutdated(mod: Mod): boolean {
	if (mod.versions.length === 0) {
		return false;
	}

	return mod.version !== mod.versions[0].name;
}

export function communityUrl(backend: Backend, author: string, mod?: string) {
	if (backend === Backend.Hexium) {
		return `https://mods.valtools.org/${mod === undefined ? `teams/${author}` : `mods/1/${author}/${mod}`}`;
	} else {
		return `https://thunderstore.io/c/${games.active?.slug}/p/${author}${mod && `/${mod}`}/`;
	}
}

export function modIconSrc(mod: Mod) {
	if (mod.type === 'remote') {
		if (mod.backend === Backend.Thunderstore) {
			let fullName = `${mod.author}-${mod.name}-${mod.version}`;
			return thunderstoreIconUrl(fullName);
		} else {
			return hexiumIconUrl('' + mod.author, mod.name);
		}
	} else if (mod.icon !== null) {
		let path = mod.enabled === false ? mod.icon + '.old' : mod.icon;
		return convertFileSrc(path);
	} else {
		return `games/${games.active?.slug}.webp`;
	}
}

export function gameIconSrc(game: Game) {
	return `https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/images/games/${game.slug}.webp`;
}

export function thunderstoreIconUrl(fullName: string) {
	return `https://gcdn.thunderstore.io/live/repository/icons/${fullName}.png`;
}

export function hexiumIconUrl(pkg: string, name: string) {
	return `https://mods.valtools.org/uploads/${pkg}/${name}/icon.png`;
}

export function capitalize(str: string): string {
	return str.charAt(0).toLocaleUpperCase() + str.slice(1);
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
	return /^#?([0-9A-Fa-f]{6})$/.test(str);
}

export function discordAvatarUrl(discordId: string, avatar: string) {
	return `https://cdn.discordapp.com/avatars/${discordId}/${avatar}.png`;
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

export async function getMarkdown(mod: Mod, type: MarkdownType, useLatest = false) {
	switch (mod.type) {
		case ModType.Remote:
			return await api.thunderstore.getMarkdown(
				{
					packageUuid: mod.uuid,
					versionUuid: useLatest ? mod.versions[0].uuid : mod.versionUuid,
					backend: mod.backend
				},
				type
			);

		case ModType.Local:
			return await api.profile.getLocalMarkdown(mod.uuid, type);
	}
}
