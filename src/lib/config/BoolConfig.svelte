<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';

	export let entryId: ConfigEntryId;
	export let locked: boolean;

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
	<Checkbox bind:value={content} {onValueChanged} disabled={locked} />
</div>
<ResetConfigButton {entryId} {locked} {onReset} />
