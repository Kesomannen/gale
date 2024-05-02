<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import EnumConfig from '$lib/config/EnumConfig.svelte';
	import StringConfig from '$lib/config/StringConfig.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type {
		ConfigEntry,
		ConfigEntryId,
		ConfigFile,
		ConfigSection,
		ConfigValue,
		GetConfigResult
	} from '$lib/models';
	import { Tooltip } from 'bits-ui';
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { fileName, sentenceCase } from '$lib/util';
	import BoolConfig from '$lib/config/BoolConfig.svelte';
	import SliderConfig from '$lib/config/SliderConfig.svelte';
	import { Render } from '@jill64/svelte-sanitize';
	import FlagsConfig from '$lib/config/FlagsConfig.svelte';
	import Icon from '@iconify/svelte';
	import NumberInputConfig from '$lib/config/NumberInputConfig.svelte';
	import { currentProfile } from '$lib/profile';

	let files: GetConfigResult[] | undefined;

	let searchTerm: string | undefined;

	let selectedFile: ConfigFile | undefined;
	let selectedSection: ConfigSection | undefined;

	$: {
		$currentProfile;
		files = undefined;
		refresh();
	}

	$: shownFiles = searchTerm?.length ?? 0 > 1
		? files!.filter((file) => fileName(file).toLowerCase().includes(searchTerm!.toLowerCase()))
		: files

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
				return 'Bool';
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
		files = await invokeCommand<GetConfigResult[]>('get_config_files');
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div
		class="flex flex-col py-4 min-w-60 w-[25%] gap-1 bg-gray-700 border-r border-gray-600 overflow-y-auto overflow-x-hidden"
	>
		{#if files === undefined}
			<div class="flex items-center justify-center w-full h-full text-slate-300 text-lg">
				<Icon icon="mdi:loading" class="animate-spin mr-2" />
				Loading config...
			</div>
		{:else if files.length === 0}
			<div class="text-center mt-auto mb-auto text-slate-300 text-lg">
				No config files found
			</div>
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
				{#if file.type == 'ok'}
					<ConfigFileTreeItem
						file={file.content}
						{selectedSection}
						onClick={(file, section) => {
							selectedFile = file;
							selectedSection = section;
						}}
						onDeleted={(_) => refresh()}
					/>
				{:else}
					<Tooltip.Root openDelay={100}>
						<Tooltip.Trigger
							class="flex items-center text-white bg-red-600 pl-3 pr-2 cursor-default"
						>
							<Icon icon="mdi:error" class="mr-2 flex-shrink-0" />
							<div class="flex-shrink truncate">
								{file.content.file}
							</div>
						</Tooltip.Trigger>
						<Tooltip.Content
							class="rounded-lg bg-gray-800 border border-gray-600 text-slate-300 px-4 py-2 max-w-[30rem] shadow-lg"
							transition={fly}
							transitionConfig={{ duration: 100 }}
							side="right"
						>
							<Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
							{file.content.error}
						</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			{/each}
		{/if}
	</div>

	{#if selectedFile && selectedSection}
		<div class="flex flex-col flex-grow p-4 gap-1 overflow-y-auto">
			<h1 class="text-slate-200 text-lg font-semibold pb-1 truncate flex-shrink-0">
				{selectedFile.name}
				<span class="text-slate-400">/</span>
				{selectedSection.name}
			</h1>

			{#each selectedSection.entries as entry (entry.name)}
				<div class="flex items-center text-slate-300 pl-1 h-7">
					<Tooltip.Root openDelay={200}>
						<Tooltip.Trigger
							class="text-slate-300 mr-auto pr-2 cursor-auto w-[50%] text-left truncate flex-shrink-0"
						>
							{sentenceCase(entry.name)}
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
								<span class="font-semibold text-slate-200 text-md">{entry.name}</span>
								<span class="text-slate-400"> ({typeName(entry)})</span>
							</div>

							<Render html={entry.description.replace(/\n/g, '<br/>')} />
							{#if entry.defaultValue}
								<p class="mt-1">
									<span class="font-bold">Default: </span>
									{configValueToString(entry.defaultValue)}
								</p>
							{/if}
						</Tooltip.Content>
					</Tooltip.Root>
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
			{/each}
		</div>
	{/if}
</div>
