<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import { Button } from 'bits-ui';

	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { expandedEntry } from './ExpandedEntryPopup.svelte';
	import { getListSeparator } from '$lib/util';

	export let entryId: ConfigEntryId;
	export let isOther: boolean = false;

	let content = entryId.entry.value.content as string;
	let listSeparator = getListSeparator(entryId.entry);

	async function onReset(value: ConfigValue) {
		content = value.content as string;
		await submit();
	}

	async function submit() {
		await setConfigEntry(entryId, {
			type: isOther ? 'other' : 'string',
			content
		});
	}

	$: showExpandButton =
		content.length > 100 ||
		listSeparator.type === 'custom' ||
		content.includes('\n') ||
		content.includes(listSeparator.char);
</script>

<div class="flex-grow relative">
	<InputField bind:value={content} on:change={submit} class="w-full {showExpandButton && 'pr-8'}" />

	{#if showExpandButton}
		<Button.Root
			class="absolute right-1 top-1 p-1 text-slate-400 text-lg rounded-lg hover:bg-gray-800 bg-gray-900"
			on:click={() => ($expandedEntry = entryId)}
		>
			<Icon icon="mdi:arrow-expand" />
		</Button.Root>
	{/if}
</div>
<ResetConfigButton {entryId} {onReset} />
