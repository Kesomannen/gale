<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigSection, LoadFileResult } from '$lib/models';
	import { capitalize } from '$lib/util';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile } from '$lib/stores';
	import { page } from '$app/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfigEntryField from '$lib/config/ConfigEntryField.svelte';
	import ConfigFileEditor from '$lib/config/ConfigFileEditor.svelte';

	let files: LoadFileResult[] | undefined;

	let searchTerm = '';

	let selectedFile: LoadFileResult | undefined;
	let selectedSection: ConfigSection | undefined;

	$: {
		$activeProfile;
		files = undefined;
		selectedFile = undefined;
		selectedSection = undefined;
		refresh();
	}

	$: shownFiles = sortAndFilterFiles(searchTerm, files ?? []);

	function sortAndFilterFiles(searchTerm: string, files: LoadFileResult[]) {
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
		files = await invokeCommand<LoadFileResult[]>('get_config_files');

		let searchParam = $page.url.searchParams.get('file');
		if (searchParam) {
			selectedFile = files.find((file) => file.relativePath === searchParam);
			if (selectedFile === undefined) {
				return;
			}

			if (selectedFile.type === 'ok') {
				selectedSection = selectedFile.sections[0];
			}

			searchTerm = selectedFile.relativePath;
		}

		$page.url.searchParams.delete('file');
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div
		class="file-list w-[20%] min-w-72 overflow-hidden overflow-y-auto border-r border-gray-600 bg-gray-700"
	>
		{#if files === undefined}
			<div class="flex h-full w-full items-center justify-center text-lg text-slate-300">
				<Icon icon="mdi:loading" class="mr-4 animate-spin" />
				Loading config...
			</div>
		{:else if files.length === 0}
			<div class="flex h-full items-center justify-center text-lg text-slate-300">
				No config files found
			</div>
		{:else}
			<div class="relative mx-2 my-2">
				<SearchBar bind:value={searchTerm} placeholder="Search for files..." brightness={800} />
			</div>

			{#each shownFiles ?? [] as file (file.relativePath)}
				<ConfigFileTreeItem
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

	<div class="flex-grow overflow-y-auto p-4">
		{#if selectedFile !== undefined}
			<div class="flex-shrink-0 truncate text-2xl font-bold text-slate-200">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="font-light text-slate-400">/</span>
					{selectedSection.name}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor file={selectedFile} section={selectedSection} />
			{:else if selectedFile.type === 'unsupported'}
				<div class="mb-1 text-slate-400">
					This file is in an unsupported format. Please open it in an external program to make
					changes.
				</div>
				<BigButton
					color="gray"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					Open in external program
				</BigButton>
			{:else if selectedFile.type === 'err'}
				<div class="mb-1 text-slate-400">An error occured while reading this config file:</div>
				<code class="mb-1 flex bg-gray-900 p-3 text-red-500">
					{capitalize(selectedFile.error)}
				</code>
				<BigButton
					color="gray"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					Open in external program
				</BigButton>
			{/if}
		{:else}
			<div class="flex h-full w-full items-center justify-center text-lg text-slate-400">
				Select a config file to start editing
			</div>
		{/if}
	</div>
</div>

<ExpandedEntryPopup />

<style lang="postcss">
	.file-list {
		scrollbar-color: theme(colors.gray.400) theme(colors.gray.700);
	}
</style>
