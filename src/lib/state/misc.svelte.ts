import type { ConfigEntryId } from '$lib/types';

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
