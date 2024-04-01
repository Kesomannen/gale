<script lang="ts">
	import InputField from '$lib/InputField.svelte';
	import { setConfig } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';

	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;
    export let isOther: boolean = false;

	let content = entryId.entry.value.content as string;

	function onReset(value: ConfigValue) {
		content = value.content as string;
	}

	$: setConfig(entryId, {
		type: isOther ? 'other' : 'string',
		content: content
	});
</script>

<InputField bind:value={content} />
<ResetConfigButton {entryId} {onReset} />
