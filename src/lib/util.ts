import {
	type Mod,
	type ConfigEntry,
	type SyncUser,
	type Game,
	type MarkdownType,
	ModType
} from './types';
import { convertFileSrc } from '@tauri-apps/api/core';
import games from './state/game.svelte';
import { isLatinAlphabet } from './i18n';
import { m } from './paraglide/messages';
import * as api from '$lib/api';

export function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

function pluralize(str: string): string {
	return isLatinAlphabet(str) ? str + 's' : str;
}

export function formatModName(name: string): string {
	return name.replace(/_/g, ' ');
}

export function formatTime(seconds: number): string {
	if (seconds < 60) {
		return m.util_formatTime_seconds({ seconds: Math.round(seconds) });
	}

	if (seconds < 3600) {
		let minutes = Math.floor(seconds / 60);
		if (minutes > 1) {
			return pluralize(m.util_formatTime_minute({ minutes: minutes }));
		}
		return m.util_formatTime_minute({ minutes: minutes });
	}

	let hours = Math.floor(seconds / 3600);
	if (hours > 1) {
		return pluralize(m.util_formatTime_hour({ hours: hours }));
	}
	return m.util_formatTime_hour({ hours: hours });
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
		if (interval >= 1) return [interval, m.util_timeSince_year()];

		interval = Math.floor(seconds / (60 * 60 * 24 * 30));
		if (interval >= 1) return [interval, m.util_timeSince_month()];

		interval = Math.floor(seconds / (60 * 60 * 24 * 7));
		if (interval >= 1) return [interval, m.util_timeSince_week()];

		interval = Math.floor(seconds / (60 * 60 * 24));
		if (interval >= 1) return [interval, m.util_timeSince_day()];

		interval = Math.floor(seconds / (60 * 60));
		if (interval >= 1) return [interval, m.util_timeSince_hour()];

		interval = Math.floor(seconds / 60);
		if (interval >= 1) return [interval, m.util_timeSince_minute()];

		return [null, null];
	})();

	if (!interval || !str) {
		return '';
	}

	switch (interval) {
		case null:
			return m.util_timeSince_interval_null();
		case 1:
			return m.util_timeSince_interval_1({ str });
		default:
			return m.util_timeSince_interval_default({ interval, str });
	}
}

export function isOutdated(mod: Mod): boolean {
	if (mod.versions.length === 0) {
		return false;
	}

	return mod.version !== mod.versions[0].name;
}

export function communityUrl(path: string) {
	return `https://thunderstore.io/c/${games.active?.slug}/p/${path}/`;
}

export function modIconSrc(mod: Mod) {
	if (mod.type === 'remote') {
		let fullName = `${mod.author}-${mod.name}-${mod.version}`;
		return thunderstoreIconUrl(fullName);
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

export function capitalize(str: string): string {
	if (!isLatinAlphabet(str)) return str;

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
					versionUuid: useLatest ? mod.versions[0].uuid : mod.versionUuid
				},
				type
			);

		case ModType.Local:
			return await api.profile.getLocalMarkdown(mod.uuid, type);
	}
}
