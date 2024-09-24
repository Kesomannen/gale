<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type {
		ConfigEntry,
		ConfigEntryId,
		ConfigSection,
		ConfigValue,
		LoadFileResult
	} from '$lib/models';
	import { capitalize, sentenceCase } from '$lib/util';
	import BoolConfig from '$lib/config/BoolConfig.svelte';
	import SliderConfig from '$lib/config/SliderConfig.svelte';
	import FlagsConfig from '$lib/config/FlagsConfig.svelte';
	import ExpandedEntryPopup from '$lib/config/ExpandedEntryPopup.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import EnumConfig from '$lib/config/EnumConfig.svelte';
	import NumberInputConfig from '$lib/config/NumberInputConfig.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile } from '$lib/stores';
	import { Render } from '@jill64/svelte-sanitize';
	import { page } from '$app/stores';
	import StringConfig from '$lib/config/StringConfig.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import { readFile } from '@tauri-apps/plugin-fs';
	import Dropdown from '$lib/components/Dropdown.svelte';

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

	function configValueToString(config: ConfigValue) {
		switch (config.type) {
			case 'boolean':
				return config.content ? 'True' : 'False';
			case 'string':
				return config.content;
			case 'double':
			case 'int32':
			case 'single':
				return config.content.value.toString();
			case 'enum':
				return config.content.options[config.content.index];
			case 'flags':
				return config.content.indicies.map((i) => config.content.options[i]).join(', ');
			case 'other':
				return config.content;
		}
	}

	function typeName(config: ConfigEntry) {
		switch (config.value.type) {
			case 'int32':
				return 'Integer';
			case 'double':
			case 'single':
				return 'Decimal';
			case 'string':
				return 'String';
			case 'boolean':
				return 'Boolean';
			default:
				return config.typeName;
		}
	}

	function isNum(config: ConfigValue) {
		return config.type === 'int32' || config.type === 'double' || config.type === 'single';
	}

	function entryId(entry: ConfigEntry): ConfigEntryId {
		return {
			file: selectedFile!,
			section: selectedSection!,
			entry
		};
	}

	async function refresh() {
		files = await invokeCommand<LoadFileResult[]>('get_config_files');
		console.log(files);

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
				{#if selectedFile.metadata}
					<div class="font-medium text-slate-400">
						Created by {selectedFile.metadata.pluginName}
						{selectedFile.metadata.pluginVersion}
					</div>
				{/if}

				{#if selectedSection !== undefined}
					{#each selectedSection.entries as entry (entry)}
						{#if entry.type === 'normal'}
							<div class="my-1 flex items-center pl-2 text-slate-300">
								<Tooltip
									side="top"
									class="w-[45%] min-w-52 flex-shrink-0 cursor-auto truncate pr-2 text-left text-slate-300"
								>
									{sentenceCase(entry.name)}
									<svelte:fragment slot="tooltip">
										<div>
											<span class="text-lg font-bold text-slate-200">{entry.name}</span>
											<span class="ml-1 text-slate-400"> ({typeName(entry)})</span>
										</div>

										<div class="mb-1">
											<Render html={entry.description.replace(/\n/g, '<br/>')} />
										</div>

										{#if entry.defaultValue}
											<p>
												<span class="font-semibold">Default: </span>
												{configValueToString(entry.defaultValue)}
											</p>
										{/if}

										{#if (entry.value.type === 'int32' || entry.value.type === 'double' || entry.value.type === 'single') && entry.value.content.range}
											<p>
												<span class="font-semibold">Range: </span>
												{entry.value.content.range.start} - {entry.value.content.range.end}
											</p>
										{/if}
									</svelte:fragment>
								</Tooltip>
								{#if entry.value.type === 'string'}
									<StringConfig entryId={entryId(entry)} />
								{:else if entry.value.type === 'enum'}
									<EnumConfig entryId={entryId(entry)} />
								{:else if entry.value.type === 'flags'}
									<FlagsConfig entryId={entryId(entry)} />
								{:else if entry.value.type === 'boolean'}
									<BoolConfig entryId={entryId(entry)} />
								{:else if entry.value.type == 'other'}
									<StringConfig entryId={entryId(entry)} isOther={true} />
								{:else if isNum(entry.value)}
									{#if entry.value.content.range}
										<SliderConfig entryId={entryId(entry)} />
									{:else}
										<NumberInputConfig entryId={entryId(entry)} />
									{/if}
								{/if}
							</div>
						{/if}
					{/each}
				{/if}
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
