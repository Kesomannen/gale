<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';

	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import { expandedEntry } from '$lib/components/dialogs/ExpandedConfigEntryDialog.svelte';
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
		onchange={submit}
		spellcheck="false"
		disabled={locked}
		class="w-full {showExpandButton && 'pr-8'}"
	/>

	{#if showExpandButton && !locked}
		<button
			class="bg-primary-900 text-primary-400 hover:bg-primary-800 absolute top-1 right-1 rounded-lg p-1 text-lg"
			onclick={() => ($expandedEntry = entryId)}
		>
			<Icon icon="mdi:arrow-expand" />
		</button>
	{/if}
</div>
<ResetConfigButton {entryId} {locked} {onReset} />
