<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';

	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let content = entryId.entry.value.content as string;
	let hasHashtag = content.startsWith('#');

	let hexCode = $state(hasHashtag ? content : `#${content}`);

	async function onReset(value: ConfigValue) {
		hexCode = hasHashtag ? (value.content as string) : `#${value.content}`;
		await submit();
	}

	async function submit() {
		content = hasHashtag ? hexCode : hexCode.slice(1);

		await setConfigEntry(entryId, {
			type: 'string',
			content
		});
	}
</script>

<input type="color" class="grow" disabled={locked} bind:value={hexCode} onchange={submit} />
<ResetConfigButton {entryId} {onReset} {locked} />
