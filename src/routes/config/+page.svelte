<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type {
		TaggedConfigEntry,
		ConfigEntryId,
		ConfigSection,
		ConfigValue,
		LoadFileResult,
		ConfigEntry
	} from '$lib/models';
	import { capitalize, fileName, sentenceCase } from '$lib/util';
	import BoolConfig from '$lib/config/BoolConfig.svelte';
	import SliderConfig from '$lib/config/SliderConfig.svelte';
	import FlagsConfig from '$lib/config/FlagsConfig.svelte';
	import Icon from '@iconify/svelte';
	import { currentProfile } from '$lib/profile';
	import { Tooltip } from 'bits-ui';
	import { Render } from '@jill64/svelte-sanitize';
	import { fly } from 'svelte/transition';
	import StringConfig from '$lib/config/StringConfig.svelte';
	import EnumConfig from '$lib/config/EnumConfig.svelte';
	import NumberInputConfig from '$lib/config/NumberInputConfig.svelte';
	import UntaggedConfig from '$lib/config/UntaggedConfig.svelte';

	let files: LoadFileResult[] | undefined;

	let searchTerm: string | undefined;

	let selectedFile: LoadFileResult | undefined;
	let selectedSection: ConfigSection | undefined;

	$: {
		$currentProfile;
		files = undefined;
		refresh();
	}

	$: shownFiles =
		searchTerm?.length ?? 0 > 1
			? files!.filter((file) => fileName(file).toLowerCase().includes(searchTerm!.toLowerCase()))
			: files;

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
				return config.content.value;
			case 'flags':
				return config.content.values.join(', ');
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
				return 'Bool';
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
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div
		class="flex flex-col py-4 min-w-60 w-[25%] bg-gray-700 border-r border-gray-600 overflow-y-auto overflow-x-hidden"
	>
		{#if files === undefined}
			<div class="flex items-center justify-center w-full h-full text-slate-300 text-lg">
				<Icon icon="mdi:loading" class="animate-spin mr-2" />
				Loading config...
			</div>
		{:else if files.length === 0}
			<div class="text-center mt-auto mb-auto text-slate-300 text-lg">No config files found</div>
		{:else}
			<div class="relative mx-2 mb-2">
				<input
					type="text"
					class="w-full py-1.5 pr-10 pl-10 rounded-md bg-gray-800 text-slate-200 truncate"
					bind:value={searchTerm}
					placeholder="Search for files..."
				/>
				<Icon class="absolute left-[10px] top-[9px] text-slate-300 text-xl" icon="mdi:magnify" />
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

	<div class="flex flex-col flex-grow p-4 overflow-y-auto">
		{#if selectedFile}
			<div class="text-slate-200 text-lg font-semibold truncate flex-shrink-0">
				{selectedFile.content.name}
				{#if selectedSection}
					<span class="text-slate-400">/</span>
					{selectedSection.name}
				{/if}
			</div>

			{#if selectedSection && selectedFile.type === 'ok'}
				{#if selectedFile.content.metadata}
					<div class="text-slate-400 text-sm">
						Created by {selectedFile.content.metadata.pluginName} v{selectedFile.content.metadata.pluginVersion} 
					</div>
				{/if}

				<div class="h-1 flex-shrink-0" />

				{#each selectedSection.entries as entry (entry.content)}
					{#if entry.type === 'untagged'}
						<div class="flex items-center text-slate-300 pl-2 h-7 mb-1">
							<div
								class="text-slate-300 mr-auto pr-2 cursor-auto w-[50%] text-left truncate flex-shrink-0"
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
						<div class="flex items-center text-slate-300 pl-2 h-7 mb-1">
							<Tooltip.Root openDelay={200}>
								<Tooltip.Trigger
									class="text-slate-300 mr-auto pr-2 cursor-auto w-[50%] text-left truncate flex-shrink-0"
								>
									{sentenceCase(entry.content.name)}
								</Tooltip.Trigger>
								<Tooltip.Content
									class="rounded-lg bg-gray-800 border border-gray-600 text-slate-300 px-4 py-2 max-w-[35rem] shadow-lg"
									transition={fly}
									transitionConfig={{ duration: 150 }}
									side="top"
									sideOffset={2}
								>
									<Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
									<div>
										<span class="font-semibold text-slate-200 text-md">{entry.content.name}</span>
										<span class="text-slate-400"> ({typeName(entry.content)})</span>
									</div>

									<Render html={entry.content.description.replace(/\n/g, '<br/>')} />

									<div class="h-1" />

									{#if entry.content.defaultValue}
										<p>
											<span class="font-bold">Default: </span>
											{configValueToString(entry.content.defaultValue)}
										</p>
									{/if}

									{#if (entry.content.value.type === 'int32' || entry.content.value.type === 'double' || entry.content.value.type === 'single') && entry.content.value.content.range}
										<p>
											<span class="font-bold">Range: </span>
											{entry.content.value.content.range.start} - {entry.content.value.content.range.end}
										</p>
									{/if}
								</Tooltip.Content>
							</Tooltip.Root>
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
								{#if entry.content.value.content.range}
									<SliderConfig entryId={entryId(entry.content)} />
								{:else}
									<NumberInputConfig entryId={entryId(entry.content)} />
								{/if}
							{/if}
						</div>
					{/if}
				{/each}
			{:else if selectedFile.type === 'err'}
				<code class="text-red-400 bg-gray-900 px-2 py-1 rounded-md">
					{capitalize(selectedFile.content.error)}
				</code>
			{/if}
		{/if}
	</div>
</div>
