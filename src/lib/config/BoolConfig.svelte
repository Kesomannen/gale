<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let content = $state(entryId.entry.value.content as boolean);

	function onReset(newValue: ConfigValue) {
		content = newValue.content as boolean;
	}

	function onValueChanged(newValue: boolean) {
		content = newValue;
		setConfigEntry(entryId, { type: 'bool', content });
	}
</script>

<div class="flex grow items-center">
	<Checkbox bind:checked={content} onCheckedChange={onValueChanged} disabled={locked} />
</div>
<ResetConfigButton {entryId} {locked} {onReset} />
