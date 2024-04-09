export type ConfigValue =
	| { type: 'boolean'; content: boolean }
	| { type: 'string'; content: string }
	| {
			type: 'enum';
			content: {
				value: string;
				options: string[];
			};
	  }
	| {
			type: 'flags';
			content: {
				values: string[];
				options: string[];
			};
	  }
	| { type: 'int32'; content: ConfigNum }
	| { type: 'single'; content: ConfigNum }
	| { type: 'double'; content: ConfigNum }
	| { type: 'other'; content: string };

export interface ConfigEntry {
	name: string;
	description: string;
	typeName: string;
	defaultValue?: ConfigValue;
	value: ConfigValue;
}

export interface ConfigSection {
	name: string;
	entries: ConfigEntry[];
}

export interface ConfigFile {
	name: string;
	sections: ConfigSection[];
}

export interface ConfigNum {
	value: number;
	range?: ConfigRange;
}

export interface ConfigRange {
	start: number;
	end: number;
}

export type GetConfigResult =
	| { type: 'ok', content: ConfigFile }
	| { type: 'error', content: {
		file: string;
		error: string;
	} };

export type PrefValue =
	| { type: 'Path'; content: string }
	| { type: 'OptionPath'; content?: string }
	| { type: 'Bool'; content: boolean };

export interface PrefEntry {
	name: string;
	value: PrefValue;
}

export interface ProfileInfo {
	activeIndex: number;
	names: string[];
}

export interface GameInfo {
	active: Game;
	all: Game[];
}

export interface Mod {
	name: string;
	description?: string,
	categories?: string[],
	version?: string,
	author?: string,
	rating?: number,
	downloads?: number,
	websiteUrl?: string,
	donateUrl?: string,
	icon?: string,
	dependencies?: string[],
	isPinned: boolean,
	uuid: string;
	latestVersionUuid?: string;
	type: 'local' | 'remote';
}

export enum SortBy {
	LastUpdated = 'lastUpdated',
	Downloads = 'downloads',
	Rating = 'rating'
}

export interface QueryModsArgs {
	page: number;
	pageSize: number;
	searchTerm?: string;
	categories: string[];
	includeNsfw: boolean;
	includeDeprecated: boolean;
	sortBy: SortBy;
	descending: boolean;
}

export interface SelectItem {
	value: string;
	label: string;
}

export interface ConfigEntryId {
	file: ConfigFile;
	section: ConfigSection;
	entry: ConfigEntry;
}

export interface DropdownOption {
	icon?: string;
	label: string;
	onClick: () => void;
}

export interface Dependant {
	name: string;
	uuid: string;
}

export type RemoveModResponse = 
	| { type: "removed", content?: undefined }
	| { type: "hasDependants", content: Dependant[] };

export type InstallTask = 
	| { type: "installing", content?: undefined }
	| { type: "extracting", content?: undefined }
	| { type: "downloading", content: {
		total: number;
		downloaded: number;
	} };

export interface InstallProgress {
	installedMods: number;
	totalMods: number;
	downloadedBytes: number;
	totalBytes: number;
	currentModName: string;
	currentTask: InstallTask;
}

export type InstallProgressPayload = 
	| { type: "inProgress", content: InstallProgress }
	| { type: "done", content?: undefined }
	| { type: "error", content?: undefined };

export interface ModpackArgs {
	name: string;
	description: string;
	versionNumber: string;
	icon: string;
	websiteUrl?: string;
}

export interface Game {
	name: string,
	displayName: string,
	steamId: number,
}
