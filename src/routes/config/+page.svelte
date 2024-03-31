<script lang="ts">
	import ConfigFileTreeItem from '$lib/config/ConfigFileTreeItem.svelte';
	import EnumConfig from '$lib/config/EnumConfig.svelte';
	import StringConfig from '$lib/config/StringConfig.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigEntry, ConfigFile, ConfigSection, ConfigValue, GetConfigResult } from '$lib/models';
	import { Button, Collapsible, Slider, Tooltip } from 'bits-ui';
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { pascalToSentence } from '$lib/util';
	import BoolConfig from '$lib/config/BoolConfig.svelte';
	import SliderConfig from '$lib/config/SliderConfig.svelte';
	import { Render } from '@jill64/svelte-sanitize';
	import FlagsConfig from '$lib/config/FlagsConfig.svelte';
	import Icon from '@iconify/svelte';

	let files: GetConfigResult[] = [];

	let selectedFile: ConfigFile | undefined;
	let selectedSection: ConfigSection | undefined;

	onMount(async () => {
		files = await invokeCommand<GetConfigResult[]>('get_config_files');
	});

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
            case 'int32': return 'Integer'
            case 'double': return 'Decimal'
            case 'single': return 'Decimal'
            case 'string': return 'String'
            case 'boolean': return 'Bool'
			default: return config.typeName;
		}
	}

	function isNum(config: ConfigValue) {
		return config.type === 'int32' || config.type === 'double' || config.type === 'single';
	}
</script>

<div class="flex flex-grow overflow-hidden">
	<div
		class="flex flex-col py-4 min-w-40 w-[25%] gap-1 bg-gray-700 text-white border-r border-gray-600 overflow-y-auto overflow-x-hidden"
	>
		{#each files as file}
			{#if file.type == 'ok'}
				<ConfigFileTreeItem
					file={file.content}
					{selectedSection}
					onClick={(file, section) => {
						selectedFile = file;
						selectedSection = section;
					}}
				/>
			{:else}
				<Tooltip.Root openDelay={100}>
					<Tooltip.Trigger
						class="flex items-center text-slate bg-red-600 text-left px-4 cursor-default"
					>
						<Icon icon="mdi:alert" class="mr-2" />
						{file.content.file}
					</Tooltip.Trigger>
					<Tooltip.Content
						class="rounded-lg bg-gray-800 border border-gray-600 text-slate-300 px-4 py-2 max-w-[30rem] shadow-lg"
						transition={fly}
						transitionConfig={{ duration: 100 }}
						side="right"
					>
						{file.content.error}
					</Tooltip.Content>
				</Tooltip.Root>
			{/if}
		{/each}
	</div>

	{#if selectedFile && selectedSection}
		<div class="flex flex-col flex-grow p-4 gap-1 overflow-y-auto">
			<h1 class="text-slate-200 text-lg font-semibold pb-1 truncate">
				{selectedFile.name}
				<span class="text-slate-400">/</span>
				{selectedSection.name}
			</h1>

			{#each selectedSection.entries as entry}
				<div class="flex items-center text-slate-300 pl-1 h-7">
					<Tooltip.Root openDelay={200}>
						<Tooltip.Trigger
							class="text-slate-300 mr-auto pr-2 cursor-auto w-72 text-left truncate flex-shrink-0"
						>
							{pascalToSentence(entry.name)}
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
						<StringConfig file={selectedFile.name} section={selectedSection.name} {entry} />
					{:else if entry.value.type === 'enum'}
						<EnumConfig file={selectedFile.name} section={selectedSection.name} {entry} />
					{:else if entry.value.type === 'flags'}
						<FlagsConfig file={selectedFile.name} section={selectedSection.name} {entry} />
					{:else if entry.value.type === 'boolean'}
						<BoolConfig file={selectedFile.name} section={selectedSection.name} {entry} />
					{:else if isNum(entry.value)}
						{#if entry.value.content.range === undefined}
							PLACEHOLDER
						{:else}
							<SliderConfig file={selectedFile.name} section={selectedSection.name} {entry} />
						{/if}
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
