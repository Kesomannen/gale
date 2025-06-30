<script lang="ts">
	import { run } from 'svelte/legacy';

	import ConfigFileListItem from '$lib/config/ConfigFileListItem.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigSection, ConfigFile } from '$lib/models';
	import { capitalize } from '$lib/util';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile, activeProfileLocked } from '$lib/stores';
	import { page } from '$app/stores';
	import BigButton from '$lib/components/Button.svelte';
	import ConfigFileEditor from '$lib/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/modlist/ProfileLockedBanner.svelte';

	let files: ConfigFile[] | null = $state();

	let searchTerm = $state('');

	let selectedFile: ConfigFile | null = $state();
	let selectedSection: ConfigSection | null = $state();

	function sortAndFilterFiles(searchTerm: string, files: ConfigFile[]) {
		if (searchTerm.length > 0) {
			files = files.filter((file) => {
				let lowerSearch = searchTerm.toLowerCase().trim();

				return (
					file.relativePath.toLowerCase().includes(lowerSearch) ||
					file.displayName?.toLowerCase().includes(lowerSearch)
				);
			});
		}

		files.sort((a, b) => {
			return (a.displayName ?? a.relativePath).localeCompare(b.displayName ?? b.relativePath);
		});

		return files;
	}

	async function refresh() {
		files = await invokeCommand<ConfigFile[]>('get_config_files');

		let searchParam = $page.url.searchParams.get('file');
		if (searchParam === null) return;

		selectedFile = files.find((file) => file.relativePath === searchParam) ?? null;
		if (selectedFile === null) return;

		if (selectedFile.type === 'ok') {
			selectedSection = selectedFile.sections[0];
		}

		searchTerm = selectedFile.relativePath;
		$page.url.searchParams.delete('file');
	}
	run(() => {
		$activeProfile;
		files = null;
		selectedFile = null;
		selectedSection = null;
		refresh();
	});
	let shownFiles = $derived(sortAndFilterFiles(searchTerm, files ?? []));
</script>

<div class="flex grow overflow-hidden">
	<div
		class="light-scrollbar border-primary-600 bg-primary-700 w-[20%] min-w-72 overflow-hidden overflow-y-auto border-r"
	>
		{#if files === null}
			<div class="text-primary-300 flex h-full w-full items-center justify-center text-lg">
				<Icon icon="mdi:loading" class="mr-4 animate-spin" />
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
					onDeleted={() => {
						refresh();
						selectedFile = null;
					}}
				/>
			{/each}
		{/if}
	</div>

	<div class="flex max-w-4xl grow flex-col overflow-y-auto py-4">
		{#if $activeProfileLocked}
			<ProfileLockedBanner class="mx-4 mb-4" />
		{/if}

		{#if selectedFile !== null}
			<div class="shrink-0 truncate px-4 text-2xl font-bold text-white">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="text-primary-400">/</span>
					{selectedSection.name.length > 0 ? selectedSection.name : '<Nameless section>'}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor
					file={selectedFile}
					section={selectedSection}
					locked={$activeProfileLocked}
				/>
			{:else if selectedFile.type === 'unsupported'}
				<div class="text-primary-400 mb-1 px-4">
					This file is in an unsupported format. Please open it in an external program to make
					changes.
				</div>
				<BigButton
					class="mx-4 max-w-max"
					color="primary"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					Open in external program
				</BigButton>
			{:else if selectedFile.type === 'err'}
				<div class="text-primary-400 mb-1 px-4">
					An error occured while reading this config file:
				</div>
				<code class="bg-primary-900 mx-4 mb-1 flex rounded-sm p-4 text-red-500">
					{capitalize(selectedFile.error)}
				</code>
				<BigButton
					class="mx-4 max-w-max"
					color="primary"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					Open in external program
				</BigButton>
			{/if}
		{:else}
			<div class="text-primary-400 flex w-full grow items-center justify-center text-lg">
				Select a config file to start editing
			</div>
		{/if}
	</div>
</div>

<ExpandedEntryPopup />
