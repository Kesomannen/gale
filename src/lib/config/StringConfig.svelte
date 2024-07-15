<script lang="ts">
	import { setTaggedConfig } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import { Button } from 'bits-ui';

	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';
	import Popup from '$lib/components/Popup.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ResizableInputField from '$lib/components/ResizableInputField.svelte';

	export let entryId: ConfigEntryId;
	export let isOther: boolean = false;

	let content = entryId.entry.value.content as string;

	let dialogOpen = false;

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

	<Button.Root
		class="absolute right-1 top-1 p-1 text-slate-400 text-lg rounded-lg hover:bg-gray-800 bg-gray-900"
		on:click={() => (dialogOpen = true)}
	>
		<Icon icon="mdi:arrow-expand" />
	</Button.Root>
</div>
<ResetConfigButton {entryId} {onReset} />

<!--
{#if dialogOpen}
	<Popup bind:open={dialogOpen} title="Edit {entryId.entry.name}">
		<ResizableInputField bind:value={content} placeholder="Enter text..." />
	</Popup>
{/if}
-->
