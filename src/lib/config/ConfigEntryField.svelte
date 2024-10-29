<script lang="ts">
	import { sentenceCase } from '$lib/util';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import type { ConfigEntry, ConfigEntryId, ConfigValue } from '$lib/models';
	import { Render } from '@jill64/svelte-sanitize';
	import StringConfig from './StringConfig.svelte';
	import EnumConfig from './EnumConfig.svelte';
	import FlagsConfig from './FlagsConfig.svelte';
	import BoolConfig from './BoolConfig.svelte';
	import SliderConfig from './SliderConfig.svelte';
	import NumberInputConfig from './NumberInputConfig.svelte';
	import { isNum } from '$lib/config';

	export let entryId: ConfigEntryId;

	$: ({ entry } = entryId);
	$: ({ value } = entry);

	$: typeName = getTypeName(entry);

	function valueToString(val: ConfigValue) {
		switch (val.type) {
			case 'boolean':
				return val.content ? 'True' : 'False';
			case 'string':
				return val.content;
			case 'double':
			case 'int32':
			case 'single':
				return val.content.value.toString();
			case 'enum':
				return val.content.options[val.content.index];
			case 'flags':
				return val.content.indicies.map((i) => val.content.options[i]).join(', ');
			case 'other':
				return val.content;
		}
	}

	function getTypeName(entry: ConfigEntry) {
		switch (value.type) {
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
				return entry.typeName;
		}
	}
</script>

<!-- odd:bg-[#1b2433] -->
<div class="field flex items-center py-1 pl-6 pr-4 text-slate-300">
	<Tooltip
		class="w-[45%] min-w-52 flex-shrink-0 cursor-auto truncate py-1 pr-2 text-left text-slate-300"
		openDelay={50}
	>
		{sentenceCase(entry.name)}

		<svelte:fragment slot="tooltip">
			<div>
				<span class="text-lg font-semibold text-white">{entry.name}</span>
				<span class="ml-1 text-slate-400"> ({typeName})</span>
			</div>

			<div class="mb-1">
				{#if entry.description === null}
					[No description provided]
				{:else}
					<Render html={entry.description.replace(/\n/g, '<br/>')} />
				{/if}
			</div>

			{#if entry.defaultValue}
				<p class="break-words">
					<span class="font-medium text-slate-100">Default: </span>
					{valueToString(entry.defaultValue)}
				</p>
			{/if}

			{#if (value.type === 'int32' || value.type === 'double' || value.type === 'single') && value.content.range}
				<p>
					<span class="font-medium text-white">Range: </span>
					{value.content.range.start} - {value.content.range.end}
				</p>
			{/if}
		</svelte:fragment>
	</Tooltip>
	{#if value.type === 'string'}
		<StringConfig {entryId} />
	{:else if value.type === 'enum'}
		<EnumConfig {entryId} />
	{:else if value.type === 'flags'}
		<FlagsConfig {entryId} />
	{:else if value.type === 'boolean'}
		<BoolConfig {entryId} />
	{:else if value.type == 'other'}
		<StringConfig {entryId} isOther={true} />
	{:else if isNum(value)}
		{#if value.content.range}
			<SliderConfig {entryId} />
		{:else}
			<NumberInputConfig {entryId} />
		{/if}
	{/if}
</div>
