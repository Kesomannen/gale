export type ConfigValueType<T extends string, C> = { type: T; content: C };

export type ConfigValue =
	| ConfigValueType<'bool', boolean>
	| ConfigValueType<'string', string>
	| ConfigValueType<'int', ConfigNum>
	| ConfigValueType<'float', ConfigNum>
	| ConfigValueType<'enum', { index: number; options: string[] }>
	| ConfigValueType<'flags', { indicies: number[]; options: string[] }>;

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
	metadata: ConfigFileMetadata | null;
};

export type ConfigFileMetadata = {
	modName: string;
	modVersion: string;
};

export type ConfigNum = {
	value: number;
	range: ConfigRange | null;
};

export type ConfigRange = {
	start: number;
	end: number;
};

export type ConfigFileType<T extends string, C = {}> = { type: T } & C;

export type ConfigFile = { relativePath: string; displayName: string | null } & (
	| ConfigFileType<'ok', ConfigFileData>
	| ConfigFileType<'err', { error: string }>
	| ConfigFileType<'unsupported'>
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
	missing: boolean;
};

export type ListedSyncProfile = {
	id: string;
	name: string;
	community: string;
	createdAt: string;
	updatedAt: string;
};

export type SyncUser = {
	discordId: string;
	name: string;
	displayName: string;
	avatar: string | null;
};

export type ManagedGameInfo = {
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
	versions: ModVersion[];
	type: ModType;
	enabled?: boolean | null;
	icon: string | null;
	configFile: string | null;
};

export type ModVersion = {
	name: string;
	uuid: string;
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

export type SortOrder = 'ascending' | 'descending';

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
	maxCount: number;
};

export type QueryModsArgsWithoutMax = Omit<QueryModsArgs, 'maxCount'>;

export type ConfigEntryId = {
	file: { relativePath: string };
	section: ConfigSection;
	entry: ConfigEntry;
};

export type Dependant = {
	fullName: string;
	uuid: string;
};

export type ModId = {
	packageUuid: string;
	versionUuid: string;
};

export type ModActionResponse =
	| { type: 'done' }
	| { type: 'hasDependants'; dependants: Dependant[] };

export type InstallTask = 'download' | 'extract' | 'install';

export type InstallEvent =
	| { type: 'show' }
	| { type: 'hide'; reason: 'done' | 'error' | 'cancelled' }
	| { type: 'addCount'; mods: number; bytes: number }
	| { type: 'addProgress'; mods: number; bytes: number }
	| { type: 'setTask'; name: string; task: InstallTask };

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
	ReturnOfModding = 'ReturnOfModding',
	BepisLoader = 'BepisLoader'
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

export type ImportData =
	| ({ type: 'legacy' } & LegacyImportData)
	| ({ type: 'sync' } & SyncImportData);

export type LegacyImportData = {
	manifest: ProfileManifest;
	path: string;
	deleteAfterImport: boolean;
	missingMods: string[];
};

export type SyncImportData = {
	manifest: ProfileManifest;
	id: string;
	created_at: string;
	updated_at: string;
	owner: SyncUser;
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
	fetchModsAutomatically: boolean;
	pullBeforeLaunch: boolean;
	zoomFactor: number;
	gamePrefs: Map<string, GamePrefs>;
};

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string[];
	customArgsEnabled: boolean;
	launchMode: LaunchMode;
	platform: Platform | null;
};

export type Platform = 'steam' | 'epicGames' | 'oculus' | 'origin' | 'xboxStore';

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

export type Zoom = { factor: number } | { delta: number };

export type MarkdownCache = 'readme' | 'changelog';

export type LaunchOptionType =
	| 'none'
	| 'default'
	| 'application'
	| 'safemode'
	| 'multiplayer'
	| 'config'
	| 'vr'
	| 'server'
	| 'editor'
	| 'manual'
	| 'benchmark'
	| 'option1'
	| 'option2'
	| 'option3'
	| 'othervr'
	| 'openvroverlay'
	| 'osvr'
	| 'openxr'
	| { unknown: string };

export interface LaunchOption {
	name: string;
	arguments: string;
	type: LaunchOptionType;
}
