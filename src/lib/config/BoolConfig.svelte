<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';

	export let entryId: ConfigEntryId;

	let content = entryId.entry.value.content as boolean;

	function onReset(newValue: ConfigValue) {
		content = newValue.content as boolean;
	}

	function onValueChanged(newValue: boolean) {
		content = newValue;
		setConfigEntry(entryId, { type: 'bool', content });
	}
</script>

<div class="flex grow items-center">
	<Checkbox bind:value={content} {onValueChanged} />
</div>
<ResetConfigButton {entryId} {onReset} />
