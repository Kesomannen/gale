<script lang="ts">
	import ConfigFileListItem from '$lib/components/config/ConfigFileListItem.svelte';
	import type { ConfigFile } from '$lib/types';
	import SearchBar from '$lib/components/ui/SearchBar.svelte';

	import Spinner from '$lib/components/ui/Spinner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import config from '$lib/state/config.svelte';

	let searchTerm = $state('');

	let shownFiles = $derived(sortAndFilterFiles(searchTerm, config.files));

	let duplicateNames = $derived.by(() => {
		const nameCount = new Map<string, number>();

		config.files.forEach((file) => {
			const name = file.displayName;
			if (name) {
				nameCount.set(name, (nameCount.get(name) || 0) + 1);
			}
		});

		return new Set(
			Array.from(nameCount.entries())
				.filter(([_, count]) => count > 1)
				.map(([name, _]) => name)
		);
	});

	function sortAndFilterFiles(searchTerm: string, files: ConfigFile[]) {
		let sortedFiles = [...files];

		if (searchTerm.length > 0) {
			sortedFiles = sortedFiles.filter((file) => {
				let lowerSearch = searchTerm.toLowerCase().trim();

				return (
					file.relativePath.toLowerCase().includes(lowerSearch) ||
					file.displayName?.toLowerCase().includes(lowerSearch)
				);
			});
		}

		sortedFiles.sort((a, b) => {
			return (a.displayName ?? a.relativePath).localeCompare(b.displayName ?? b.relativePath);
		});

		return sortedFiles;
	}
</script>

<div class="overflow-x-hidden overflow-y-auto px-2 pb-4">
	<div class="relative my-2">
		<SearchBar bind:value={searchTerm} placeholder={m.configFileList_placeholder()} />
	</div>

	{#each shownFiles ?? [] as file (file.relativePath)}
		<ConfigFileListItem
			{file}
			duplicate={duplicateNames.has(file.displayName ?? '')}
			locked={profiles.activeLocked}
		/>
	{/each}
</div>
