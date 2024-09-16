import { invoke } from './invoke';
import type { ConfigEntry, InstallSource, Version } from './models';
import { communities, profiles } from './state/profile.svelte';

function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

function formatTime(seconds: number): string {
	var hours = Math.floor(seconds / 3600);
	var minutes = Math.floor((seconds % 3600) / 60);
	var secs = Math.floor(seconds % 60);

	return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

function shortenNum(value: number): string {
	var i = value == 0 ? 0 : Math.floor(Math.log(value) / Math.log(1000));
	if (i === 0) {
		return value.toString();
	}
	return (value / Math.pow(1000, i)).toFixed(1) + ['', 'k', 'M', 'G', 'T'][i];
}

function sentenceCase(str: string): string {
	const textcase = String(str)
		.replace(/^[^A-Za-z0-9]*|[^A-Za-z0-9]*$/g, '')
		.replace(/([a-z])([A-Z])/g, (m, a, b) => `${a}_${b.toLowerCase()}`)
		.replace(/[^A-Za-z0-9]+|_+/g, ' ')
		.toLowerCase();

	return textcase.charAt(0).toUpperCase() + textcase.slice(1);
}

function timeSince(date: Date): string {
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

function titleCase(str: string): string {
	return str.replace(/\b\w/g, (l) => l.toUpperCase());
}

function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1);
}

function isBefore(el1: HTMLElement, el2: HTMLElement) {
	return el1.compareDocumentPosition(el2) === Node.DOCUMENT_POSITION_PRECEDING;
}

export interface ListSeparator {
	type: 'default' | 'custom';
	char: string;
}

function getListSeparator(entry: ConfigEntry): ListSeparator {
	const keyword = 'ListSeparator=';

	let description = entry.description;
	let separatorIndex = description.indexOf(keyword);

	if (separatorIndex !== -1) {
		return { type: 'custom', char: description[separatorIndex + keyword.length] };
	}

	return { type: 'default', char: ',' };
}

function modIdentifier(owner: string, name: string, version: Version) {
	return `${owner}-${name}-${version.major}.${version.minor}.${version.patch}`;
}

function modIconUrl(owner: string, name: string, version: Version) {
	return `https://gcdn.thunderstore.io/live/repository/icons/${modIdentifier(owner, name, version)}.png`;
}

function modThunderstoreUrl(owner: string, name: string) {
	return `https://thunderstore.io/c/${communities.active?.slug}/p/${owner}/${name}/`;
}

async function queueInstall(source: InstallSource) {
	await invoke('profile', 'queue_install', { source, profileId: profiles.activeId })
}

async function queueThunderstoreInstall(
	owner: string,
	name: string,
	version: { id: string } & Version
){
	await queueInstall({
		type: 'thunderstore',
		identifier: modIdentifier(owner, name, version),
		versionUuid: version.id
	})	
}

export { 
	modIconUrl,
	modThunderstoreUrl,
	shortenFileSize,
	formatTime,
	shortenNum,
	sentenceCase,
	timeSince,
	titleCase,
	capitalize,
	isBefore,
	getListSeparator,
	queueInstall,
	queueThunderstoreInstall,
	modIdentifier
};
