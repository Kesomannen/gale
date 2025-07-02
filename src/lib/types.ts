export type ConfigValue =
	| { type: 'bool'; content: boolean }
	| { type: 'string'; content: string }
	| { type: 'int'; content: ConfigNum }
	| { type: 'float'; content: ConfigNum }
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
	default: ConfigValue | null;
	value: ConfigValue;
};

export type ConfigSection = {
	name: string;
	entries: ConfigEntry[];
};

export type ConfigFileData = {
	displayName: string;
	relativePath: string;
	sections: ConfigSection[];
	metadata: {
		modName: string;
		modVersion: string;
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

export type ConfigFile = { relativePath: string; displayName: string | null } & (
	| ({ type: 'ok' } & ConfigFileData)
	| { type: 'unsupported' }
	| {
			type: 'err';
			error: string;
	  }
);

export type ProfileInfo = {
	id: number;
	name: string;
	modCount: number;
	sync: SyncProfileInfo | null;
};

export type SyncProfileInfo = {
	id: string;
	owner: SyncUser;
	syncedAt: string;
	updatedAt: string;
};

export type ListedSyncProfile = {
	id: String;
	name: String;
	community: string;
	createdAt: string;
	updatedAt: string;
};

export type SyncUser = {
	discordId: string;
	name: string;
	displayName: string;
	avatar: string;
};

export type SyncImportData = {
	id: string;
	created_at: string;
	updated_at: string;
	owner: SyncUser;
	manifest: ProfileManifest;
};

export type ProfilesInfo = {
	profiles: ProfileInfo[];
	activeId: number;
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
	versionUuid: string;
	lastUpdated: string | null;
	versions: {
		name: string;
		uuid: string;
	}[];
	type: ModType;
	enabled?: boolean | null;
	icon: string | null;
	configFile: string | null;
};

export enum ModType {
	Local = 'local',
	Remote = 'remote'
}

export type SortBy =
	| 'newest'
	| 'name'
	| 'author'
	| 'lastUpdated'
	| 'downloads'
	| 'rating'
	| 'installDate'
	| 'custom'
	| 'diskSpace';

export type SortOrder =
	| 'ascending'
	| 'descending';

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
	durationSecs: number;
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
	modLoader: ModLoader;
	popular: boolean;
};

export enum ModLoader {
	BepInEx = 'BepInEx',
	MelonLoader = 'MelonLoader',
	Northstar = 'Northstar',
	GDWeave = 'GDWeave',
	ReturnOfModding = 'ReturnOfModding'
}

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
	totalModCount: number;
	unknownMods: Dependant[];
	updates: AvailableUpdate[];
};

export type AnyImportData = ({ type: 'normal' } & ImportData) | ({ type: 'sync' } & SyncImportData);

export type ImportData = {
	manifest: ProfileManifest;
	path: string;
	deleteAfterImport: boolean;
};

type ProfileManifest = {
	profileName: string;
	mods: ProfileManifestMod[];
	community: string | null;
	ignoredUpdates: string[];
};

type ProfileManifestMod = {
	name: string;
	enabled: string;
	version: {
		major: number;
		minor: number;
		patch: number;
	};
};

export type R2ImportData = {
	path: string;
	profiles: string[];
	include: boolean[];
};

export type Prefs = {
	dataDir: string;
	cacheDir: string;
	sendTelemetry: boolean;
	fetchModsAutomatically: boolean;
	pullBeforeLaunch: boolean;
	zoomFactor: number;
	gamePrefs: Map<string, GamePrefs>;
};

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string[] | null;
	launchMode: LaunchMode;
	platform: Platform | null;
};

export type Platform =
	| 'steam'
	| 'epicGames'
	| 'oculus'
	| 'origin'
	| 'xboxStore';


export type ContextItem = {
	label: string;
	icon?: string;
	onclick: () => void;
	children?: ContextItem[];
};

export type ModContextItem = {
	label: string;
	icon?: string;
	showFor?: (mod: Mod, locked: boolean) => boolean;
	onclick: (mod: Mod) => void;
	children?: (mod: Mod) => ModContextItem[];
};
