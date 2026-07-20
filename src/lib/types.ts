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

export type BaseConfigFile = { relativePath: string; displayName: string | null };

export type ConfigFile = BaseConfigFile &
	(
		| ConfigFileType<'ok', ConfigFileData>
		| ConfigFileType<'err', { error: string }>
		| ConfigFileType<'unsupported'>
	);

export type ProfileInfo = {
	id: number;
	name: string;
	modCount: number;
	sync: SyncProfileInfo | null;
	customArgs: string;
	missing: boolean;
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
	lastUpdated: string;
	all: Game[];
	favorites: string[];
};

export enum Backend {
	Thunderstore = 'Thunderstore',
	Hexium = 'Hexium'
}

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
	suggestions: string[] | null;
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
	backend: Backend;
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
	maxCount: number | null;
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
	backend: Backend;
};

export type DependantWithVersion = {
	fullName: string;
	preferredVersion: string | null;
	backend: Backend;
};

export type ModId = {
	packageUuid: string;
	versionUuid: string;
	backend: Backend;
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

export type FetchEvent =
	| { type: 'start'; backend: Backend }
	| { type: 'progress'; mods: number }
	| { type: 'done'; backend: Backend };

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
	backend: Backend;
};

export type ModpackInfo = {
	args: ModpackArgs;
	hexiumExclusive: boolean;
};

export type ExportCode = {
	code: string;
	backend: Backend;
};

export type Game = {
	name: string;
	slug: string;
	platforms: Platform[];
	favorite: boolean;
	modLoader: ModLoader;
	popular: boolean;
	backends: Backend[];
};

export enum ModLoader {
	BepInEx = 'BepInEx',
	MelonLoader = 'MelonLoader',
	Northstar = 'Northstar',
	GDWeave = 'GDWeave',
	ReturnOfModding = 'ReturnOfModding',
	BepisLoader = 'BepisLoader',
	Shimloader = 'Shimloader',
	Lovely = 'Lovely'
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
	backend: Backend;
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
	source: Backend;
};

export type R2ImportData = {
	path: string;
	profiles: string[];
	include: boolean[];
};

export type ImportOptions = {
	importAll?: boolean;
	merge?: boolean;
};

export type Prefs = {
	dataDir: string;
	cacheDir: string;
	fetchModsAutomatically: boolean;
	pullBeforeLaunch: boolean;
	zoomFactor: number;
	language: string;
	gamePrefs: Map<string, GamePrefs>;
	backendSkipConfirm: boolean;
};

export enum Backends {
	All = 'All',
	Thunderstore = 'Thunderstore',
	Hexium = 'Hexium'
}

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string;
	launchMode: LaunchMode;
	platform: Platform | null;
	backend: Backends;
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

export type MarkdownType = 'readme' | 'changelog';

export type MissingProfileAction = { type: 'locate'; newPath: string } | { type: 'delete' };

export type Folder = {
	id: string;
	children: ListItem[];
};

export type ListItem =
	| {
			type: 'mod';
			mod: Mod;
	  }
	| {
			type: 'folder';
			folder: Folder;
	  };
