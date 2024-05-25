<script lang="ts">
	import { setTaggedConfig } from '$lib/invoke';
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
		setTaggedConfig(entryId, { type: 'boolean', content });
	}
</script>

<div class="flex items-center flex-grow">
	<Checkbox bind:value={content} {onValueChanged} />
</div>
<ResetConfigButton {entryId} {onReset} />
