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

export interface ConfigEntry {
	name: string;
	description: string;
	typeName: string;
	defaultValue?: ConfigValue;
	value: ConfigValue;
}

export interface ConfigSection {
	name: string;
	entries: ({ type: 'orphaned' } | ({ type: 'normal' } & ConfigEntry))[];
}

export interface ConfigFile {
	displayName: string;
	relativePath: string;
	metadata?: {
		pluginName: string;
		pluginVersion: string;
		pluginGuid: string;
	};
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
	| ({ type: 'ok' } & ConfigFile)
	| { type: 'unsupported'; relativePath: string; displayName: null }
	| {
			type: 'err';
			displayName: string;
			relativePath: string;
			error: string;
	  };

export enum SortBy {
	Newest = 'newest',
	Name = 'name',
	Author = 'author',
	LastUpdated = 'lastUpdated',
	Downloads = 'downloads',
	Rating = 'rating',
	InstallDate = 'installDate',
	Custom = 'custom',
	DiskSpace = 'diskSpace'
}

export enum SortOrder {
	Ascending = 'ascending',
	Descending = 'descending'
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
	entry: ConfigEntry;
}

export interface Dependant {
	name: string;
	uuid: string;
}

export type ModActionResponse =
	| { type: 'done'; content?: undefined }
	| { type: 'hasDependants' | 'hasDependencies'; content: Dependant[] };

export type InstallTask =
	| { kind: 'done'; payload?: undefined }
	| { kind: 'error'; payload?: undefined }
	| { kind: 'installing'; payload?: undefined }
	| { kind: 'extracting'; payload?: undefined }
	| {
			kind: 'downloading';
			payload: {
				total: number;
				downloaded: number;
			};
	  };

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
	author: string;
	categories: string[];
	nsfw: boolean;
	readme: string;
	changelog: string;
	versionNumber: string;
	iconPath: string;
	websiteUrl: string;
	includeDisabled: boolean;
	includeFileMap: Map<string, boolean>;
}

export interface Game {
	id: string;
	displayName: string;
	aliases: string[];
	steamId: number;
	favorite: boolean;
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
	| { type: 'steam'; content?: undefined }
	| { type: 'direct'; content: { instances: number; intervalSecs: number } };

export interface AvailableUpdate {
	fullName: string;
	ignore: boolean;
	packageUuid: string;
	versionUuid: string;
	old: string;
	new: string;
}

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

export interface Prefs {
	steamExePath?: string;
	steamLibraryDir?: string;
	dataDir: string;
	cacheDir: string;
	enableModCache: boolean;
	fetchModsAutomatically: boolean;
	zoomFactor: number;
	gamePrefs: Map<string, GamePrefs>;
}

export interface GamePrefs {
	dirOverride?: string;
	launchMode: LaunchMode;
}

export type ProfileInfo = {
	id: number;
	name: string;
	path: string;
	communityId: number;
	mods: ProfileModInfo[];
};

export type ProfileModInfo = {
	id: number;
	index: number;
	name: string;
	version: string | null;
	enabled: boolean;
	href: string;
	kind: ProfileModKind;
};

export type ProfileModKind = 'thunderstore' | 'local' | 'github';

export type GameInfo = {
	id: number;
	name: string;
	slug: string;
	isFavorite: boolean;
};

export type InstallSource =
	| { type: 'thunderstore', identifier: string, versionUuid: string }
	| { type: 'local', path: string, fullName: string, version: string }
	| { type: 'github', owner: string, repo: string, tag: string };

export type Version = {
	major: number;
	minor: number;
	patch: number;
}

export type ModImport = {
	source: InstallSource;
	enabled: boolean;
}

export type ImportData = {
	mods: ModImport[];
	sourcePath: string;
	deleteAfterImport: boolean;
}

export type ImportTarget =
	| { type: 'new', name: string, path: string, communityId: number }
	| { type: 'overwrite', id: number };
