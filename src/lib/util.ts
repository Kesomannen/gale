import type { Mod, ConfigEntry } from './models';
import { get } from 'svelte/store';
import { t } from '$i18n';

export function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

export function formatTime(seconds: number): string {
	var hours = Math.floor(seconds / 3600);
	var minutes = Math.floor((seconds % 3600) / 60);
	var secs = Math.floor(seconds % 60);

	return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
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

export function timeSince(date: Date): string {
	var seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
	var interval = Math.floor(seconds / 31536000);

	if (interval > 1) {
		return interval + ` ${get(t)["years"]}`;
	}
	interval = Math.floor(seconds / 2592000);
	if (interval > 1) {
		return interval + ` ${get(t)["months"]}`;
	}
	interval = Math.floor(seconds / 86400);
	if (interval > 1) {
		return interval + ` ${get(t)["days"]}`;
	}
	interval = Math.floor(seconds / 3600);
	if (interval > 1) {
		return interval + ` ${get(t)["hours"]}`;
	}
	interval = Math.floor(seconds / 60);
	if (interval > 1) {
		return interval + ` ${get(t)["minutes"]}`;
	}
	return Math.floor(seconds) + ` ${get(t)["seconds"]}`;
}

export function titleCase(str: string): string {
	return str.replace(/\b\w/g, (l) => l.toUpperCase());
}

export function isOutdated(mod: Mod): boolean {
	if (mod.versions.length === 0) {
		return false;
	}

	return mod.version !== mod.versions[0].name;
}

export function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1);
}

export function isBefore(el1: HTMLElement, el2: HTMLElement) {
	return el1.compareDocumentPosition(el2) === Node.DOCUMENT_POSITION_PRECEDING;
}

export interface ListSeparator {
	type: 'default' | 'custom';
	char: string;
}

export function getListSeparator(entry: ConfigEntry): ListSeparator {
	const keyword = 'ListSeparator=';

	let description = entry.description;
	let separatorIndex = description.indexOf(keyword);

	if (separatorIndex !== -1) {
		return { type: 'custom', char: description[separatorIndex + keyword.length] };
	}

	return { type: 'default', char: ',' };
}
