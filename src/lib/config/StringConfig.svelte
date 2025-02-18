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

	let content = entryId.entry.value.content as string;
	let listSeparator = getListSeparator(entryId.entry);

	async function onReset(value: ConfigValue) {
		content = value.content as string;
		await submit();
	}

	async function submit() {
		await setConfigEntry(entryId, {
			type: 'string',
			content
		});
	}

	$: showExpandButton =
		content.length > 100 ||
		listSeparator.type === 'custom' ||
		content.includes('\n') ||
		content.includes(listSeparator.char);
</script>

<div class="relative grow">
	<InputField
		bind:value={content}
		on:change={submit}
		spellcheck="false"
		class="w-full {showExpandButton && 'pr-8'}"
	/>

	{#if showExpandButton}
		<Button.Root
			class="absolute right-1 top-1 rounded-lg bg-slate-900 p-1 text-lg text-slate-400 hover:bg-slate-800"
			on:click={() => ($expandedEntry = entryId)}
		>
			<Icon icon="mdi:arrow-expand" />
		</Button.Root>
	{/if}
</div>
<ResetConfigButton {entryId} {onReset} />
