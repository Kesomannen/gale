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

export interface PackageVersion {
	dateCreated: string;
	dependencies: string[];
	description: string;
	downloadUrl: string;
	downloads: number;
	fileSize: number;
	fullName: string;
	icon: string;
	isActive: boolean;
	name: string;
	uuid4: string;
	versionNumber: string;
	websiteUrl: string;
}

export interface PackageListing {
	categories: string[];
	dateCreated: string;
	dateUpdated: string;
	donationLink?: string;
	fullName: string;
	hasNsfwContent: boolean;
	isDeprecated: boolean;
	isPinned: boolean;
	name: string;
	owner: string;
	packageUrl: string;
	ratingScore: number;
	uuid4: string;
	versions: PackageVersion[];
}

export interface LegacyProfileCreateResponse {
	key: string;
}

export interface PackageInstaller {
	identifier: string;
}

export interface PackageManifest {
	name: string;
	description: string;
	versionNumber: string;
	dependencies: string[];
	websiteUrl: string;
	installers?: PackageInstaller[];
}

export interface Mod {
	package: PackageListing;
	version: PackageVersion;
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
	sortBy?: SortBy;
	descending: boolean;
}

export interface SelectItem {
	value: string;
	label: string;
}
