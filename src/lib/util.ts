import type { GetConfigResult } from "./models";

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

export function sentenceCase(str: string): string {
	const textcase = String(str)
		.replace(/^[^A-Za-z0-9]*|[^A-Za-z0-9]*$/g, '')
		.replace(/([a-z])([A-Z])/g, (m, a, b) => `${a}_${b.toLowerCase()}`)
		.replace(/[^A-Za-z0-9]+|_+/g, ' ')
		.toLowerCase();

	return textcase.charAt(0).toUpperCase() + textcase.slice(1);
}

export function fileName(configFile: GetConfigResult) {
	if (configFile.type == "ok") {
		return configFile.content.name;
	}

	return configFile.content.file;
}
