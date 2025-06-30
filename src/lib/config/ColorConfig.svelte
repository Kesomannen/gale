<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';

	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	// we don't use hashtags in cfg files, since they are turned into comments
	// but the color input works with the # prefix, so we have to separate them
	let content = entryId.entry.value.content as string;
	let hexCode = $state(`#${content}`);

	async function onReset(value: ConfigValue) {
		hexCode = `#${value.content}`;
		await submit();
	}

	async function submit() {
		content = hexCode.slice(1);

		await setConfigEntry(entryId, {
			type: 'string',
			content
		});
	}
</script>

<input type="color" class="grow" disabled={locked} bind:value={hexCode} onchange={submit} />
<ResetConfigButton {entryId} {onReset} {locked} />
