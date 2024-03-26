export interface PackageVersion {
	date_created: string;
	dependencies: string[];
	description: string;
	download_url: string;
	downloads: number;
	file_size: number;
	full_name: string;
	icon: string;
	is_active: boolean;
	name: string;
	uuid4: string;
	version_number: string;
	website_url: string;
}

export interface PackageListing {
	categories: string[];
	date_created: string;
	date_updated: string;
	donation_link: string | undefined;
	full_name: string;
	has_nsfw_content: boolean;
	is_deprecated: boolean;
	is_pinned: boolean;
	name: string;
	owner: string;
	package_url: string;
	rating_score: number;
	uuid4: string;
	versions: PackageVersion[];
}

export interface Mod {
	package: PackageListing;
	version: PackageVersion;
}

export type SortBy = "LastUpdated" | "Downloads" | "Rating";

export interface ModQueryArgs {
	page: number,
	page_size: number,
	search_term: string | undefined,
	categories: string[],
	include_nsfw: boolean,
	include_deprecated: boolean,
	sort_by: undefined | SortBy,
	descending: boolean,
}

export type ConfigType = 'Path' | 'OptionPath' | 'Bool';

export interface ConfigValue {
	name: string;
	value: {
		type: ConfigType;
		content: any;
	};
}