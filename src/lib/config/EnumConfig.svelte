<script lang="ts">
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import { setConfigEntry } from '$lib/config';
	import Dropdown from '$lib/components/Dropdown.svelte';

	export let entryId: ConfigEntryId;
	export let locked: boolean;

	let content = entryId.entry.value.content as { index: number; options: string[] };
	let selected = content.options[content.index];

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { index: number; options: string[] };
		selected = content.options[content.index];
	}

	function onSelectedChange(value: string) {
		let index = content.options.indexOf(value);
		setConfigEntry(entryId, {
			type: 'enum',
			content: {
				index,
				options: content.options
			}
		});
	}
</script>

<Dropdown
	items={content.options}
	class="grow"
	bind:selected
	{onSelectedChange}
	multiple={false}
	disabled={locked}
/>
<ResetConfigButton {entryId} {onReset} {locked} />
