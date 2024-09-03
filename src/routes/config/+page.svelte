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

	import { get } from 'svelte/store';
	import { T, t } from '$i18n';

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
				return get(t)['Integer'];
			case 'double':
			case 'single':
				return get(t)['Decimal'];
			case 'string':
				return get(t)['String'];
			case 'boolean':
				return get(t)['Boolean'];
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
		class="file-list overflow-y-auto min-w-72 w-[20%] bg-gray-700 overflow-hidden border-r border-gray-600"
	>
		{#if files === undefined}
			<div class="flex items-center justify-center w-full h-full text-slate-300 text-lg">
				<Icon icon="mdi:loading" class="animate-spin mr-4" />
				{get(t)["Loading config"]}
			</div>
		{:else if files.length === 0}
			<div class="text-center mt-auto mb-auto text-slate-300 text-lg">{get(t)['No config files']}</div>
		{:else}
			<div class="relative mx-2 my-2">
				<SearchBar bind:value={searchTerm} placeholder="{get(t)["Search for files"]}" brightness={800} />
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

	<div class="flex-grow p-4 overflow-y-auto">
		{#if selectedFile !== undefined}
			<div class="text-slate-200 text-2xl font-bold truncate flex-shrink-0">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="text-slate-400 font-light">/</span>
					{selectedSection.name}
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				{#if selectedFile.metadata}
					<div class="text-slate-400 font-medium">
						{T(get(t)["Config created by"], {
							"pluginName": selectedFile.metadata.pluginName,
							"pluginVersion": selectedFile.metadata.pluginVersion
						})}
					</div>
				{/if}

				{#if selectedSection !== undefined}
					{#each selectedSection.entries as entry (entry)}
						{#if entry.type === 'normal'}
							<div class="flex items-center text-slate-300 pl-2 my-1">
								<Tooltip
									side="top"
									class="w-[45%] min-w-52 text-slate-300 pr-2 cursor-auto text-left truncate flex-shrink-0"
								>
									{sentenceCase(entry.name)}
									<svelte:fragment slot="tooltip">
										<div>
											<span class="text-slate-200 text-lg font-bold">{entry.name}</span>
											<span class="text-slate-400 ml-1"> ({typeName(entry)})</span>
										</div>

										<div class="mb-1">
											<Render html={entry.description.replace(/\n/g, '<br/>')} />
										</div>

										{#if entry.defaultValue}
											<p>
												<span class="font-semibold">{get(t)["Default"]}: </span>
												{configValueToString(entry.defaultValue)}
											</p>
										{/if}

										{#if (entry.value.type === 'int32' || entry.value.type === 'double' || entry.value.type === 'single') && entry.value.content.range}
											<p>
												<span class="font-semibold">{get(t)["Range"]}: </span>
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
				<div class="text-slate-400 mb-1">
					{get(t)["Config unsupported format"]}
				</div>
				<BigButton
					color="gray"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					{get(t)["Open in external program"]}
				</BigButton>
			{:else if selectedFile.type === 'err'}
				<div class="text-slate-400 mb-1">{get(t)["Error reading config file"]}</div>
				<code class="flex text-red-500 bg-gray-900 px-2 py-1 mb-1 rounded">
					{capitalize(selectedFile.error)}
				</code>
				<BigButton
					color="gray"
					on:click={() => invokeCommand('open_config_file', { file: selectedFile?.relativePath })}
				>
					<Icon icon="mdi:open-in-new" class="mr-2" />
					{get(t)["Open in external program"]}
				</BigButton>
			{/if}
		{:else}
			<div class="flex items-center justify-center text-lg text-slate-400 w-full h-full">
				{get(t)["Select config file editing"]}
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
