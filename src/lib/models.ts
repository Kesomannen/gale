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

export type ConfigEntry =
  | { type: 'tagged', content: TaggedConfigEntry }
	| { type: 'untagged', content: { name: string; value: string; } };

export interface TaggedConfigEntry {
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
	metadata?: {
		pluginName: string;
		pluginVersion: string;
		pluginGuid: string;
	}
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

export type LoadFileResult =
	| { type: 'ok', content: ConfigFile }
	| { type: 'err', content: {
		name: string;
		error: string;
	} };

export type PrefValue = string | LaunchMode;
	
export interface ProfileInfo {
	activeIndex: number;
	names: string[];
}

export interface GameInfo {
	active: Game;
	all: Game[];
	favorites: string[];
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
	isDeprecated: boolean,
	uuid: string;
	versions: {
		name: string,
		uuid: string
	}[];
	type: 'local' | 'remote';
	enabled: boolean;
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
	includeDisabled: boolean;
	sortBy: SortBy;
	descending: boolean;
}

export interface SelectItem {
	value: string;
	label: string;
}

export interface ConfigEntryId {
	file: LoadFileResult;
	section: ConfigSection;
	entry: TaggedConfigEntry;
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

export type ModActionResponse = 
	| { type: "done", content?: undefined }
	| { type: "hasDependants", content: Dependant[] }
	| { type: "hasDependencies", content: Dependant[] }

export type InstallTask = 
	| { kind: "done", payload?: undefined }
	| { kind: "error", payload?: undefined }
	| { kind: "installing", payload?: undefined }
	| { kind: "extracting", payload?: undefined }
	| { kind: "downloading", payload: {
		total: number;
		downloaded: number;
	} };

export interface InstallProgress {
	totalProgress: number;
	installedMods: number;
	totalMods: number;
	currentName: string;
	task: InstallTask;
}

export interface ModpackArgs {
	name: string;
	description: string;
	versionNumber: string;
	icon: string;
	websiteUrl?: string;
}

export interface Game {
	id: string,
	displayName: string,
	steamId: number,
	favorite: boolean,
}

export interface PackageCategory {
	id: string;
	name: string;
	slug: string;
}

export interface FiltersResponse {
	package_categories: PackageCategory[];
}

export type LaunchMode = 
	| { type: 'steam', content?: undefined }
	| { type: 'direct', content: { instances: number } };

export interface AvailableUpdate {
	name: string;
	uuid: string;
	old: string;
	new: string;
}

export interface ProfileQuery {
	mods: Mod[];
	updates: AvailableUpdate[];
}

export interface ImportData {
	name: string;
	temp_path: string;
	mods: {
		enabled: boolean;
		packageUuid: string;
		versionUuid: string;
	}[]
};