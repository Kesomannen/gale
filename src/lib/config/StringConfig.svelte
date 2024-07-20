<script lang="ts">
	import { setTaggedConfig } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import { Button } from 'bits-ui';

	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { expandedEntry } from './ExpandedEntryPopup.svelte';

	export let entryId: ConfigEntryId;
	export let isOther: boolean = false;

	let content = entryId.entry.value.content as string;

	function onReset(value: ConfigValue) {
		content = value.content as string;
	}

	$: setTaggedConfig(entryId, {
		type: isOther ? 'other' : 'string',
		content: content
	});
</script>

<div class="flex-grow relative">
	<InputField bind:value={content} class="flex-grow" />

	{#if content.length > 100 || content.includes(',') || content.includes('\n')}
		<Button.Root
			class="absolute right-1 top-1 p-1 text-slate-400 text-lg rounded-lg hover:bg-gray-800 bg-gray-900"
			on:click={() => ($expandedEntry = entryId)}
		>
			<Icon icon="mdi:arrow-expand" />
		</Button.Root>
	{/if}
</div>
<ResetConfigButton {entryId} {onReset} />
