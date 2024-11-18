<script lang="ts">
	import { sentenceCase } from '$lib/util';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
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

	$: typeName = getTypeName(value);

	function valueToString(val: ConfigValue) {
		switch (val.type) {
			case 'bool':
				return val.content ? 'True' : 'False';
			case 'string':
				return val.content;
			case 'int':
			case 'float':
				return val.content.value.toString();
			case 'enum':
				return val.content.options[val.content.index];
			case 'flags':
				return val.content.indicies.map((i) => val.content.options[i]).join(', ');
		}
	}

	function getTypeName(value: ConfigValue) {
		switch (value.type) {
			case 'int':
				return 'Integer';
			case 'float':
				return 'Decimal';
			case 'string':
				return 'String';
			case 'bool':
				return 'Boolean';
			case 'enum':
				return 'Enum';
			case 'flags':
				return 'Flags';
		}
	}
</script>

<!-- odd:bg-[#1b2433] -->
<div class="flex items-center py-0.5 pl-6 pr-4 text-slate-300">
	<Tooltip
		class="w-[45%] min-w-52 flex-shrink-0 cursor-auto truncate pr-2 text-left text-slate-300"
		openDelay={50}
	>
		{sentenceCase(entry.name)}

		<svelte:fragment slot="tooltip">
			<h4>
				<span class="text-lg font-semibold text-white">{entry.name}</span>
				<span class="ml-1 text-slate-400"> ({typeName})</span>
			</h4>

			{#if entry.description !== null}
				<p class="mb-1">
					{entry.description}
				</p>
			{/if}

			{#if entry.default}
				<p class="break-words">
					<span class="font-medium text-slate-100">Default: </span>
					{valueToString(entry.default)}
				</p>
			{/if}

			{#if (value.type === 'int' || value.type === 'float') && value.content.range !== null}
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
	{:else if value.type === 'bool'}
		<BoolConfig {entryId} />
	{:else if isNum(value)}
		{#if value.content.range !== null}
			<SliderConfig {entryId} />
		{:else}
			<NumberInputConfig {entryId} />
		{/if}
	{/if}
</div>
