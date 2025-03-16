<script lang="ts">
	import ConfigFileListItem from '$lib/config/ConfigFileListItem.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigSection, ConfigFile } from '$lib/models';
	import { capitalize } from '$lib/util';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile } from '$lib/stores';
	import { page } from '$app/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfigFileEditor from '$lib/config/ConfigFileEditor.svelte';

	let files: ConfigFile[] | undefined;

	let searchTerm = '';

	let selectedFile: ConfigFile | undefined;
	let selectedSection: ConfigSection | undefined;

	$: {
		$activeProfile;
		files = undefined;
		selectedFile = undefined;
		selectedSection = undefined;
		refresh();
	}

	$: shownFiles = sortAndFilterFiles(searchTerm, files ?? []);

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

		selectedFile = files.find((file) => file.relativePath === searchParam);
		if (selectedFile === undefined) return;

		if (selectedFile.type === 'ok') {
			selectedSection = selectedFile.sections[0];
		}

		searchTerm = selectedFile.relativePath;
		$page.url.searchParams.delete('file');
	}
</script>

<div class="flex grow overflow-hidden">
	<div
		class="light-scrollbar border-primary-600 bg-primary-700 w-[20%] min-w-72 overflow-hidden overflow-y-auto border-r"
	>
		{#if files === undefined}
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
					onFileClicked={(file) => {
						selectedFile = file;
						selectedSection = undefined;
					}}
					onSectionClicked={(file, section) => {
						selectedFile = { type: 'ok', ...file };
						selectedSection = section;
					}}
					onDeleted={() => {
						refresh();
						selectedFile = undefined;
					}}
				/>
			{/each}
		{/if}
	</div>

	<div class="max-w-4xl grow overflow-y-auto py-4">
		{#if selectedFile !== undefined}
			<div class="shrink-0 truncate px-4 text-2xl font-bold text-white">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="text-primary-400">/</span>
					{selectedSection.name.length > 0 ? selectedSection.name : '<Nameless section>'}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor file={selectedFile} section={selectedSection} />
			{:else if selectedFile.type === 'unsupported'}
				<div class="text-primary-400 mb-1 px-4">
					This file is in an unsupported format. Please open it in an external program to make
					changes.
				</div>
				<BigButton
					class="mx-4"
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
					class="mx-4"
					color="primary"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					Open in external program
				</BigButton>
			{/if}
		{:else}
			<div class="text-primary-400 flex h-full w-full items-center justify-center text-lg">
				Select a config file to start editing
			</div>
		{/if}
	</div>
</div>

<ExpandedEntryPopup />
