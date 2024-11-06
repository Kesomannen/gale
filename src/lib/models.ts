import type { Color } from './theme';

export type ConfigValue =
	| { type: 'boolean'; content: boolean }
	| { type: 'string'; content: string }
	| { type: 'int32'; content: ConfigNum }
	| { type: 'single'; content: ConfigNum }
	| { type: 'double'; content: ConfigNum }
	| { type: 'other'; content: string }
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
	  };

export type ConfigEntry = {
	name: string;
	description: string | null;
	typeName: string;
	defaultValue: ConfigValue | null;
	value: ConfigValue;
};

export type ConfigSection = {
	name: string;
	entries: ({ type: 'orphaned' } | ({ type: 'normal' } & ConfigEntry))[];
};

export type ConfigFile = {
	displayName: string;
	relativePath: string;
	sections: ConfigSection[];
	metadata: {
		pluginName: string;
		pluginVersion: string;
		pluginGuid: string;
	} | null;
};

export type ConfigNum = {
	value: number;
	range: ConfigRange | null;
};

export type ConfigRange = {
	start: number;
	end: number;
};

export type LoadFileResult =
	| ({ type: 'ok' } & ConfigFile)
	| { type: 'unsupported'; relativePath: string; displayName: null }
	| {
			type: 'err';
			displayName: string;
			relativePath: string;
			error: string;
	  };

export type ProfileInfo = {
	name: string;
	modCount: number;
};

export type ProfilesInfo = {
	profiles: ProfileInfo[];
	activeIndex: number;
};

export type GameInfo = {
	active: Game;
	all: Game[];
	favorites: string[];
};

export type Mod = {
	name: string;
	description: string | null;
	categories: string[] | null;
	version: string | null;
	author: string | null;
	rating: number | null;
	downloads: number | null;
	fileSize: number;
	websiteUrl: string | null;
	donateUrl: string | null;
	dependencies: string[] | null;
	isPinned: boolean;
	isDeprecated: boolean;
	isInstalled: boolean | undefined;
	containsNsfw: boolean;
	uuid: string;
	lastUpdated: string | null;
	versions: {
		name: string;
		uuid: string;
	}[];
	type: 'local' | 'remote';
	enabled: boolean | null;
	configFile: string | null;
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

export type QueryModsArgs = {
	searchTerm: string;
	includeCategories: string[];
	excludeCategories: string[];
	includeNsfw: boolean;
	includeDeprecated: boolean;
	includeDisabled: boolean;
	includeEnabled: boolean;
	sortBy: SortBy;
	sortOrder: SortOrder;
};

export type ConfigEntryId = {
	file: { relativePath: string };
	section: ConfigSection;
	entry: ConfigEntry;
};

export type Dependant = {
	fullName: string;
	uuid: string;
};

export type ModActionResponse =
	| { type: 'done' }
	| { type: 'hasDependants'; dependants: Dependant[] };

export type InstallTask =
	| { kind: 'done' }
	| { kind: 'error' }
	| { kind: 'installing' }
	| { kind: 'extracting' }
	| {
			kind: 'downloading';
			payload: {
				total: number;
				downloaded: number;
			};
	  };

export type InstallProgress = {
	totalProgress: number;
	installedMods: number;
	totalMods: number;
	currentName: string;
	canCancel: boolean;
	task: InstallTask;
};

export type ModpackArgs = {
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
};

export type Game = {
	name: string;
	slug: string;
	platforms: Platform[];
	favorite: boolean;
	popular: boolean;
};

export type PackageCategory = {
	name: string;
	slug: string;
};

export type FiltersResponse = {
	results: PackageCategory[];
};

export type LaunchMode =
	| { type: 'launcher'; content?: undefined }
	| { type: 'direct'; content: { instances: number; intervalSecs: number } };

export type AvailableUpdate = {
	fullName: string;
	ignore: boolean;
	packageUuid: string;
	versionUuid: string;
	old: string;
	new: string;
};

export type ProfileQuery = {
	mods: Mod[];
	unknownMods: Dependant[];
	updates: AvailableUpdate[];
};

export type ImportData = {
	name: string;
	includes: Map<string, string>;
	modNames: string[] | null;
	source: 'r2' | 'gale';
	mods: {
		modRef: {
			packageUuid: string;
			versionUuid: string;
		};
		enabled: boolean;
		index: number | null;
		overwrite: boolean;
		installTime: string | null;
	}[];
};

export type R2ImportData = {
	r2modman: {
		path: string;
		profiles: string[];
		include: boolean[];
	} | null;
	thunderstore: {
		path: string;
		profiles: string[];
		include: boolean[];
	} | null;
};

export type MarkdownResponse = {
	markdown: string | null;
	detail?: string;
};

export type Prefs = {
	steamExePath: string | null;
	steamLibraryDir: string | null;
	dataDir: string;
	cacheDir: string;
	fetchModsAutomatically: boolean;
	zoomFactor: number;
	gamePrefs: Map<string, GamePrefs>;
};

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string[] | null;
	launchMode: LaunchMode;
	platform: Platform;
};

export enum Platform {
	Steam = 'steam',
	EpicGames = 'epicGames',
	Oculus = 'oculus',
	Origin = 'origin',
	XboxGamePass = 'xboxGamePass'
}

export type ModContextItem = {
	label: string;
	icon?: string;
	showFor?: (mod: Mod) => boolean;
	onclick: (mod: Mod) => void;
	children?: (mod: Mod) => ModContextItem[];
};
