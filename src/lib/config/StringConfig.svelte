<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import { Button } from 'bits-ui';

	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { expandedEntry } from './ExpandedEntryPopup.svelte';
	import { getListSeparator } from '$lib/util';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let content = $state(entryId.entry.value.content as string);
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

	let showExpandButton = $derived(
		content.length > 100 ||
			listSeparator.type === 'custom' ||
			content.includes('\n') ||
			content.includes(listSeparator.char)
	);
</script>

<div class="relative grow">
	<InputField
		bind:value={content}
		on:change={submit}
		spellcheck="false"
		disabled={locked}
		class="w-full {showExpandButton && 'pr-8'}"
	/>

	{#if showExpandButton && !locked}
		<Button.Root
			class="bg-primary-900 text-primary-400 hover:bg-primary-800 absolute top-1 right-1 rounded-lg p-1 text-lg"
			on:click={() => ($expandedEntry = entryId)}
		>
			<Icon icon="mdi:arrow-expand" />
		</Button.Root>
	{/if}
</div>
<ResetConfigButton {entryId} {locked} {onReset} />
