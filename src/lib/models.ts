export type ConfigValue =
	| { type: 'boolean'; content: boolean }
	| { type: 'string'; content: string }
	| {
			type: 'enum';
			content: {
				index: number;
				options: string[];
			};
	  }
	| {
			type: 'flags';
			content: {
				indicies: number[];
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

export type PrefValue = string | LaunchMode | number;
	
export interface ProfileInfo {
	name: string;
	modCount: number;
}

export interface ProfilesInfo {
	profiles: ProfileInfo[];
	activeIndex: number;
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
	containsNsfw: boolean,
	uuid: string;
	lastUpdated: string;
	versions: {
		name: string,
		uuid: string
	}[];
	type: 'local' | 'remote';
	enabled: boolean;
	configFile?: string;
}

export enum SortBy {
	Newest = 'newest',
	Name = 'name',
	Author = 'author',
	LastUpdated = 'lastUpdated',
	Downloads = 'downloads',
	Rating = 'rating',
	InstallDate = 'installDate',
	Custom = 'custom',
}

export enum SortOrder {
	Ascending = 'ascending',
	Descending = 'descending',
}

export interface QueryModsArgs {
	maxCount: number;
	searchTerm: string;
	includeCategories: string[];
	excludeCategories: string[];
	includeNsfw: boolean;
	includeDeprecated: boolean;
	includeDisabled: boolean;
	sortBy: SortBy;
	sortOrder: SortOrder;
}

export interface ConfigEntryId {
	file: LoadFileResult;
	section: ConfigSection;
	entry: TaggedConfigEntry;
}

export interface Dependant {
	name: string;
	uuid: string;
}

export type ModActionResponse = 
	| { type: "done", content?: undefined }
	| { type: "hasDependants" | "hasDependencies", content: Dependant[] }

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
	canCancel: boolean;
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

export interface R2ImportData {
	r2modman?: {
		path: string;
		profiles: string[];
		include: boolean[];
	};
	thunderstore?: {
		path: string;
		profiles: string[];
		include: boolean[];
	};
}

export interface MarkdownResponse { 
	markdown?: string;
	detail?: string;
}