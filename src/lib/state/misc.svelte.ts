import type { ConfigEntryId, QueryModsArgsWithoutMax } from '$lib/types';
import { PersistedState } from 'runed';

export const apiKeyDialog = $state({
	open: false
});

export const config: {
	expandedEntry: ConfigEntryId | null;
} = $state({
	expandedEntry: null
});

export const updateBanner = $state({
	threshold: 0
});

export const modQuery = new PersistedState<QueryModsArgsWithoutMax>('modQuery', {
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: false,
	includeDeprecated: false,
	includeEnabled: false,
	includeDisabled: false,
	sortBy: 'rating',
	sortOrder: 'descending'
});

export const profileQuery = new PersistedState<QueryModsArgsWithoutMax>('profileQuery', {
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: true,
	includeDeprecated: true,
	includeEnabled: true,
	includeDisabled: true,
	sortBy: 'custom',
	sortOrder: 'descending'
});
