<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type {
		TaggedConfigEntry,
		ConfigEntryId,
		ConfigSection,
		ConfigValue,
		LoadFileResult
	} from '$lib/models';
	import { capitalize, configDisplayName, configFileName, sentenceCase } from '$lib/util';
	import BoolConfig from '$lib/config/BoolConfig.svelte';
	import SliderConfig from '$lib/config/SliderConfig.svelte';
	import FlagsConfig from '$lib/config/FlagsConfig.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';

	import Icon from '@iconify/svelte';
	import { activeProfile } from '$lib/stores';
	import { Render } from '@jill64/svelte-sanitize';
	import StringConfig from '$lib/config/StringConfig.svelte';
	import EnumConfig from '$lib/config/EnumConfig.svelte';
	import NumberInputConfig from '$lib/config/NumberInputConfig.svelte';
	import UntaggedConfig from '$lib/config/UntaggedConfig.svelte';
	import { page } from '$app/stores';

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
					configFileName(file).toLowerCase().includes(lowerSearch) ||
					configDisplayName(file).toLowerCase().includes(lowerSearch)
				);
			});
		}

		files.sort((a, b) => {
			let aName = configDisplayName(a);
			let bName = configDisplayName(b);

			return aName.localeCompare(bName);
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

	function typeName(config: TaggedConfigEntry) {
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

	function entryId(entry: TaggedConfigEntry): ConfigEntryId {
		return {
			file: selectedFile!,
			section: selectedSection!,
			entry
		};
	}

	async function refresh() {
		files = await invokeCommand<LoadFileResult[]>('get_config_files');

		let file = $page.url.searchParams.get('file');
		if (file) {
			selectedFile = files.find((f) => configFileName(f) === file);
			if (!selectedFile) {
				return;
			}

			if (selectedFile.type === 'ok') {
				selectedSection = selectedFile.content.sections[0];
			}

			searchTerm = selectedFile.content.name;
		}

		$page.url.searchParams.delete('file');
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div
		class="flex flex-col py-3 min-w-72 w-[20%] bg-gray-700 border-r border-gray-600 overflow-y-auto overflow-x-hidden"
	>
		{#if files === undefined}
			<div class="flex items-center justify-center w-full h-full text-slate-300 text-lg">
				<Icon icon="mdi:loading" class="animate-spin mr-4" />
				Loading config...
			</div>
		{:else if files.length === 0}
			<div class="text-center mt-auto mb-auto text-slate-300 text-lg">No config files found</div>
		{:else}
			<div class="relative mx-3 mb-2">
				<SearchBar bind:value={searchTerm} placeholder="Search for files..." brightness={800} />
			</div>

			{#each shownFiles ?? [] as file}
				<ConfigFileTreeItem
					{file}
					{selectedSection}
					onErrorFileClicked={(file) => {
						selectedFile = file;
						selectedSection = undefined;
					}}
					onSectionClicked={(file, section) => {
						selectedFile = {
							type: 'ok',
							content: file
						};
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

	<div class="flex-grow p-4 overflow-y-auto">
		{#if selectedFile}
			<div class="text-slate-200 text-xl font-semibold truncate flex-shrink-0">
				{selectedFile.content.name}
				{#if selectedSection}
					<span class="text-slate-400">/</span>
					{selectedSection.name}
				{/if}
			</div>

			{#if selectedSection && selectedFile.type === 'ok'}
				{#if selectedFile.content.metadata}
					<div class="text-slate-400">
						Created by {selectedFile.content.metadata.pluginName}
						{selectedFile.content.metadata.pluginVersion}
					</div>
				{/if}

				<div class="h-1 flex-shrink-0" />

				{#each selectedSection.entries as entry (entry.content)}
					{#if entry.type === 'untagged'}
						<div class="flex items-center text-slate-300 pl-2 my-1">
							<div
								class="text-slate-300 pr-2 cursor-auto w-[45%] text-left truncate flex-shrink-0"
							>
								{sentenceCase(entry.content.name)}
							</div>
							<UntaggedConfig
								file={selectedFile.content}
								section={selectedSection}
								name={entry.content.name}
								value={entry.content.value}
							/>
						</div>
					{:else}
						<div class="flex items-center text-slate-300 pl-2 my-1">
							<Tooltip
								side="top"
								class="w-[45%] min-w-52 text-slate-300 pr-2 cursor-auto text-left truncate flex-shrink-0"
							>
								{sentenceCase(entry.content.name)}
								<svelte:fragment slot="tooltip">
									<div>
										<span class="text-slate-200 text-lg font-semibold">{entry.content.name}</span>
										<span class="text-slate-400 ml-1"> ({typeName(entry.content)})</span>
									</div>

									<div class="mb-1">
										<Render html={entry.content.description.replace(/\n/g, '<br/>')} />
									</div>

									{#if entry.content.defaultValue}
										<p>
											<span class="font-semibold">Default: </span>
											{configValueToString(entry.content.defaultValue)}
										</p>
									{/if}

									{#if (entry.content.value.type === 'int32' || entry.content.value.type === 'double' || entry.content.value.type === 'single') && entry.content.value.content.range}
										<p>
											<span class="font-semibold">Range: </span>
											{entry.content.value.content.range.start} - {entry.content.value.content.range
												.end}
										</p>
									{/if}
								</svelte:fragment>
							</Tooltip>
							{#if entry.content.value.type === 'string'}
								<StringConfig entryId={entryId(entry.content)} />
							{:else if entry.content.value.type === 'enum'}
								<EnumConfig entryId={entryId(entry.content)} />
							{:else if entry.content.value.type === 'flags'}
								<FlagsConfig entryId={entryId(entry.content)} />
							{:else if entry.content.value.type === 'boolean'}
								<BoolConfig entryId={entryId(entry.content)} />
							{:else if entry.content.value.type == 'other'}
								<StringConfig entryId={entryId(entry.content)} isOther={true} />
							{:else if isNum(entry.content.value)}
								{#if entry.content.value.content.range && entry.content.value.content.range.end - entry.content.value.content.range.start <= 200}
									<SliderConfig entryId={entryId(entry.content)} />
								{:else}
									<NumberInputConfig entryId={entryId(entry.content)} />
								{/if}
							{/if}
						</div>
					{/if}
				{/each}
			{:else if selectedFile.type === 'err'}
				<code class="text-red-400 bg-gray-900 px-2 py-1 rounded-md flex">
					{capitalize(selectedFile.content.error)}
				</code>
			{/if}
		{:else}
			<div class="flex items-center justify-center text-lg text-slate-400 w-full h-full">
				Select a config file to start editing
			</div>
		{/if}
	</div>
</div>
