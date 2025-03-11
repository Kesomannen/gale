<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';

	export let entryId: ConfigEntryId;
	export let locked: boolean;

	let value = entryId.entry.value;
	let content = value.content as { indicies: number[]; options: string[] };
	let selected = content.indicies.map((index) => content.options[index]);

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { indicies: number[]; options: string[] };
		selected = content.indicies.map((index) => content.options[index]);
	}

	function onSelectedChange(newValues: string[]) {
		content.indicies = newValues.map((value) => content.options.indexOf(value));
		setConfigEntry(entryId, {
			type: 'flags',
			content
		});
	}
</script>

<Dropdown
	placeholder="Select values"
	items={content.options}
	disabled={locked}
	multiple
	{onSelectedChange}
	class="grow overflow-hidden"
	bind:selected
/>
<ResetConfigButton {entryId} {onReset} {locked} />
