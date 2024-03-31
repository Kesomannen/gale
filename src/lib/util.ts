import { invoke } from '@tauri-apps/api';

import type { PackageListing } from './models';

export function shortenFileSize(size: number): string {
	var i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

export function shortenNum(value: number): string {
	var i = value == 0 ? 0 : Math.floor(Math.log(value) / Math.log(1000));
	if (i === 0) {
		return value.toString();
	}
	return (value / Math.pow(1000, i)).toFixed(1) + ['', 'k', 'M', 'G', 'T'][i];
}

export function getTotalDownloads(pkg: PackageListing): number {
	return pkg.versions.reduce((acc, version) => acc + version.downloads, 0);
}

export function open(url: string) {
	invoke('open', { url });
}

export function pascalToSentence(pascalCase: string): string {
	const sentenceCase = pascalCase
		.replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
		.replace(/([a-z])([A-Z])/g, '$1 $2')
		.trim()
		.toLowerCase();

	return sentenceCase.charAt(0).toUpperCase() + sentenceCase.slice(1);
}
