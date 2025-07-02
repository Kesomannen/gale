<script lang="ts">
	import ConfigFileListItem from '$lib/config/ConfigFileListItem.svelte';
	import { invoke } from '$lib/invoke';
	import type { ConfigSection, ConfigFile } from '$lib/types';
	import { capitalize } from '$lib/util';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile, activeProfileLocked } from '$lib/stores.svelte';
	import { page } from '$app/state';
	import Button from '$lib/components/Button.svelte';
	import ConfigFileEditor from '$lib/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/modlist/ProfileLockedBanner.svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import { onMount } from 'svelte';

	type Props = {
		selectedFile: ConfigFile | null;
		selectedSection: ConfigSection | null;
	};

	let { selectedFile = $bindable(null), selectedSection = $bindable(null) }: Props = $props();

	let files: ConfigFile[] | null = $state(null);

	let searchTerm = $state('');

	$effect(() => {
		$activeProfile;

		files = null;
		selectedFile = null;
		selectedSection = null;
		refresh();
	});

	let shownFiles = $derived(sortAndFilterFiles(searchTerm, files ?? []));

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
		files = await invoke<ConfigFile[]>('get_config_files');

		let searchParam = page.url.searchParams.get('file');
		if (searchParam === null) return;

		selectedFile = files.find((file) => file.relativePath === searchParam) ?? null;
		if (selectedFile === null) return;

		if (selectedFile.type === 'ok') {
			selectedSection = selectedFile.sections[0];
		}

		searchTerm = selectedFile.relativePath;
		page.url.searchParams.delete('file');
	}
</script>

<div
	class="light-scrollbar border-primary-600 bg-primary-700 w-[20%] min-w-72 overflow-hidden overflow-y-auto border-r"
>
	{#if files === null}
		<div class="text-primary-300 flex h-full w-full items-center justify-center text-lg">
			<Spinner class="mr-2" />
			Loading config...
		</div>
	{:else if files.length === 0}
		<div class="text-primary-300 flex h-full items-center justify-center text-lg">
			No config files found
		</div>
	{:else}
		<div class="relative mx-2 my-2">
			<SearchBar bind:value={searchTerm} placeholder="Search for files..." brightness={800} />
		</div>

		{#each shownFiles ?? [] as file (file.relativePath)}
			<ConfigFileListItem
				{file}
				{selectedSection}
				locked={$activeProfileLocked}
				onFileClicked={(file) => {
					selectedFile = file;
					selectedSection = null;
				}}
				onSectionClicked={(file, section) => {
					selectedFile = { type: 'ok', ...file };
					selectedSection = section;
				}}
				onDeleteClicked={() => {
					refresh();
					selectedFile = null;
				}}
			/>
		{/each}
	{/if}
</div>
