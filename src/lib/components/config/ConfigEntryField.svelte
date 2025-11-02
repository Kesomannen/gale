<script lang="ts">
	import { isValidHex } from '$lib/util';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import StringConfig from './StringConfig.svelte';
	import EnumConfig from './EnumConfig.svelte';
	import FlagsConfig from './FlagsConfig.svelte';
	import BoolConfig from './BoolConfig.svelte';
	import SliderConfig from './SliderConfig.svelte';
	import NumberInputConfig from './NumberInputConfig.svelte';
	import { isNum } from '$lib/config';
	import Info from '$lib/components/ui/Info.svelte';
	import ColorConfig from './ColorConfig.svelte';
	import { toSentenceCase } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

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

	let entry = $derived(entryId.entry);
	let value = $derived(entry.value);
	let typeName = $derived(getTypeName(value));
</script>

<!-- odd:bg-[#1b2433] -->
<div class="text-primary-300 flex items-center py-0.5 pr-4 pl-6">
	<div class="text-primary-300 w-[45%] min-w-52 shrink-0 cursor-auto truncate pr-2 text-left">
		{toSentenceCase(entry.name)}
	</div>

	<Info>
		<h4>
			<span class="text-lg font-semibold text-white">{entry.name}</span>
			<span class="text-primary-400 ml-1"> ({typeName})</span>
		</h4>

		{#if entry.description}
			<p class="mb-1">
				{entry.description}
			</p>
		{/if}

		{#if entry.default}
			<p class="break-words">
				<span class="text-primary-100 font-medium">{m.configEntryField_default()}</span>
				{valueToString(entry.default)}
			</p>
		{/if}

		{#if (value.type === 'int' || value.type === 'float') && value.content.range !== null}
			<p>
				<span class="font-medium text-white">{m.configEntryField_range()}</span>
				{value.content.range.start} - {value.content.range.end}
			</p>
		{/if}
	</Info>

	{#if value.type === 'string'}
		{#if isValidHex(value.content)}
			<ColorConfig {entryId} {locked} />
		{:else}
			<StringConfig {entryId} {locked} />
		{/if}
	{:else if value.type === 'enum'}
		<EnumConfig {entryId} {locked} />
	{:else if value.type === 'flags'}
		<FlagsConfig {entryId} {locked} />
	{:else if value.type === 'bool'}
		<BoolConfig {entryId} {locked} />
	{:else if isNum(value)}
		{#if value.content.range !== null}
			<SliderConfig {entryId} {locked} />
		{:else}
			<NumberInputConfig {entryId} {locked} />
		{/if}
	{/if}
</div>
