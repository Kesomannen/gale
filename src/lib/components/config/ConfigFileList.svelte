<script lang="ts">
	import * as api from '$lib/api';

	import ConfigFileListItem from '$lib/components/config/ConfigFileListItem.svelte';
	import type { ConfigFile } from '$lib/types';
	import SearchBar from '$lib/components/ui/SearchBar.svelte';

	import { page } from '$app/state';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		selectedFile: ConfigFile | null;
	};

	let { selectedFile = $bindable(null) }: Props = $props();

	let currentProfileId: number | null = null;

	let files: ConfigFile[] | null = $state(null);

	let searchTerm = $state('');

	$effect(() => {
		// whever we switch to another profile, refresh the config list
		if (profiles.active && profiles.active.id !== currentProfileId) {
			currentProfileId = profiles.active.id;

			files = null;
			selectedFile = null;
			refresh();
		}
	});

	let shownFiles = $derived(sortAndFilterFiles(searchTerm, files ?? []));

	let duplicateNames = $derived.by(() => {
		const nameCount = new Map<string, number>();
		(files ?? []).forEach((file) => {
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

	async function refresh() {
		files = await api.config.getFiles();

		let searchParam = page.url.searchParams.get('file');
		if (searchParam === null) return;

		selectedFile = files.find((file) => file.relativePath === searchParam) ?? null;
		if (selectedFile === null) return;

		searchTerm = selectedFile.relativePath;
		page.url.searchParams.delete('file');
	}
</script>

<div class="overflow-x-hidden overflow-y-auto">
	{#if files === null}
		<div class="text-primary-300 flex h-full w-full items-center justify-center text-lg">
			<Spinner class="mr-2" />
			{m.configFileList_loading()}
		</div>
	{:else if files.length === 0}
		<div class="text-primary-300 flex h-full items-center justify-center text-lg">
			{m.configFileList_noFiles()}
		</div>
	{:else}
		<div class="relative mx-2 my-2">
			<SearchBar bind:value={searchTerm} placeholder={m.configFileList_placeholder()} />
		</div>

		{#each shownFiles ?? [] as file (file.relativePath)}
			<ConfigFileListItem
				{file}
				selected={selectedFile == file}
				duplicate={duplicateNames.has(file.displayName ?? '')}
				locked={profiles.activeLocked}
				onFileClicked={(file) => {
					selectedFile = file;
				}}
				onDeleteClicked={() => {
					refresh();
					selectedFile = null;
				}}
			/>
		{/each}
	{/if}
</div>
